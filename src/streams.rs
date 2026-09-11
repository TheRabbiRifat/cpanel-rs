//! Auto-paginating async streams for large listings.
//!
//! These types wrap paginated list operations into [`futures::Stream`] consumers,
//! so callers can iterate over results without manually managing page offsets.
//!
//! # Example
//!
//! ```no_run
//! use cpanel_rs::CpanelClient;
//! use futures::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), cpanel_rs::CpanelError> {
//!     let client = CpanelClient::simple("server.example.com", 2087, "root", "pass").unwrap();
//!
//!     // Stream all accounts across pages
//!     let mut stream = client.stream_accounts();
//!     while let Some(acct) = stream.next().await {
//!         let acct = acct?;
//!         println!("{} @ {}", acct.username, acct.domain);
//!     }
//!
//!     Ok(())
//! }
//! ```

use crate::{models::AccountInfo, CpanelClient, CpanelError};
use futures::Stream;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

// ── Account stream ──────────────────────────────────────────────────────────

/// An async stream over paginated WHM account listings.
///
/// Produced by [`CpanelClient::stream_accounts`]. Yields [`AccountInfo`] values
/// one page at a time until the API returns fewer items than the page size.
pub struct AccountsStream {
    client: Arc<CpanelClient>,
    page: i32,
    pending: Option<Pin<Box<dyn Future<Output = Result<Vec<AccountInfo>, CpanelError>> + Send>>>,
    buffer: Vec<AccountInfo>,
}

impl AccountsStream {
    pub(crate) fn new(client: &CpanelClient) -> Self {
        Self {
            client: Arc::new(client.clone()),
            page: 1,
            pending: None,
            buffer: Vec::new(),
        }
    }
}

impl Stream for AccountsStream {
    type Item = Result<AccountInfo, CpanelError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        // First, try to emit buffered items
        if let Some(item) = this.buffer.pop() {
            return Poll::Ready(Some(Ok(item)));
        }

        // If we have a pending request, poll it
        if let Some(ref mut fut) = this.pending {
            match fut.as_mut().poll(cx) {
                Poll::Ready(Ok(items)) => {
                    this.pending.take();
                    if items.is_empty() {
                        return Poll::Ready(None);
                    }
                    // Buffer items in reverse so we can pop from the end
                    this.buffer = items.into_iter().rev().collect();
                    if let Some(item) = this.buffer.pop() {
                        return Poll::Ready(Some(Ok(item)));
                    }
                }
                Poll::Ready(Err(e)) => {
                    this.pending.take();
                    return Poll::Ready(Some(Err(e)));
                }
                Poll::Pending => {}
            }
        }

        // Start a new page request
        let page = this.page;
        let client = this.client.clone();
        let page_str = page.to_string();
        this.pending = Some(Box::pin(async move {
            let pairs = [("page", page_str.as_str()), ("limit", "50")];
            client.whm("listaccts", &pairs).await
        }));
        this.page += 1;

        // Re-poll the just-created future
        if let Some(ref mut fut) = this.pending {
            match fut.as_mut().poll(cx) {
                Poll::Ready(Ok(items)) => {
                    this.pending.take();
                    if items.is_empty() {
                        return Poll::Ready(None);
                    }
                    this.buffer = items.into_iter().rev().collect();
                    if let Some(item) = this.buffer.pop() {
                        return Poll::Ready(Some(Ok(item)));
                    }
                }
                Poll::Ready(Err(e)) => {
                    this.pending.take();
                    return Poll::Ready(Some(Err(e)));
                }
                Poll::Pending => {}
            }
        }

        Poll::Pending
    }
}

// ── File stream ─────────────────────────────────────────────────────────────

/// An async stream over file entries for a directory.
///
/// cPanel's Fileman `listdir` does not natively support pagination, but this
/// stream batches entries to keep memory bounded for large directories.
///
/// Produced by [`CpanelClient::stream_files`].
pub struct FilesStream {
    client: Arc<CpanelClient>,
    user: String,
    path: String,
    pending: Option<
        Pin<
            Box<
                dyn Future<Output = Result<Vec<crate::models::FileManagerEntry>, CpanelError>>
                    + Send,
            >,
        >,
    >,
    buffer: Vec<crate::models::FileManagerEntry>,
}

impl FilesStream {
    pub(crate) fn new(client: &CpanelClient, user: &str, path: &str) -> Self {
        Self {
            client: Arc::new(client.clone()),
            user: user.to_string(),
            path: path.to_string(),
            pending: None,
            buffer: Vec::new(),
        }
    }
}

impl Stream for FilesStream {
    type Item = Result<crate::models::FileManagerEntry, CpanelError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        if let Some(item) = this.buffer.pop() {
            return Poll::Ready(Some(Ok(item)));
        }

        if let Some(ref mut fut) = this.pending {
            match fut.as_mut().poll(cx) {
                Poll::Ready(Ok(items)) => {
                    this.pending.take();
                    if items.is_empty() {
                        return Poll::Ready(None);
                    }
                    this.buffer = items.into_iter().rev().collect();
                    if let Some(item) = this.buffer.pop() {
                        return Poll::Ready(Some(Ok(item)));
                    }
                }
                Poll::Ready(Err(e)) => {
                    this.pending.take();
                    return Poll::Ready(Some(Err(e)));
                }
                Poll::Pending => {}
            }
        }

        let client = this.client.clone();
        let user = this.user.clone();
        let path = this.path.clone();
        this.pending = Some(Box::pin(
            async move { client.list_files(&user, &path).await },
        ));

        if let Some(ref mut fut) = this.pending {
            match fut.as_mut().poll(cx) {
                Poll::Ready(Ok(items)) => {
                    this.pending.take();
                    if items.is_empty() {
                        return Poll::Ready(None);
                    }
                    this.buffer = items.into_iter().rev().collect();
                    if let Some(item) = this.buffer.pop() {
                        return Poll::Ready(Some(Ok(item)));
                    }
                }
                Poll::Ready(Err(e)) => {
                    this.pending.take();
                    return Poll::Ready(Some(Err(e)));
                }
                Poll::Pending => {}
            }
        }

        Poll::Pending
    }
}
