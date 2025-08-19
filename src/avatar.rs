use super::*;
use bevy::asset::io::{AssetReader, AssetReaderError, VecReader};
use std::path::Path;

// the avatar string referenced in the profile view data links to the cdn
// https://cdn.bsky.app/img/avatar/plain/did:plc:vt545bncnkhhuhceflma2vxv/bafkreibkqsetx47ccocfse7mxdujmxgm57si5a5leerifnlwhmka2awr3u@jpeg
// however on wasm requests to the cdn are blocked by the CORS policy
// we have to get the pds host from the did and then request the avatar from there
// https://blusher.us-east.host.bsky.network/xrpc/com.atproto.sync.getBlob?did=did:plc:vt545bncnkhhuhceflma2vxv&cid=bafkreibkqsetx47ccocfse7mxdujmxgm57si5a5leerifnlwhmka2awr3u

static DIRECTORY: std::sync::LazyLock<reqwest::Client> =
    std::sync::LazyLock::new(reqwest::Client::new);

pub struct AvatarReader;

impl AssetReader for AvatarReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<VecReader, AssetReaderError> {
        let Some(url) = path.to_str() else {
            return Err(AssetReaderError::NotFound(path.into()));
        };
        let did = unsafe { url.get_unchecked(30..62) };
        let req = Compat::new(DIRECTORY
            .get(String::from("https://plc.directory/") + did)
            .send())
            .await
            .map_err(|_| AssetReaderError::NotFound(path.into()))?;
        let text = Compat::new(req
            .text())
            .await
            .map_err(|_| AssetReaderError::NotFound(path.into()))?;
        let tree: std::collections::BTreeMap<String, atrium_api::types::DataModel> =
            serde_json::from_str(&text).map_err(|_| AssetReaderError::NotFound(path.into()))?;
        let service = &tree["service"];
        let endpoint = service.get(0);
        let Ok(Some(ipld_core::ipld::Ipld::Map(pds))) = endpoint else {
            return Err(AssetReaderError::NotFound(path.into()));
        };
        let ipld_core::ipld::Ipld::String(host) = &pds["serviceEndpoint"] else {
            return Err(AssetReaderError::NotFound(path.into()));
        };
        let did = did.parse().unwrap();
        let cid = unsafe { url.get_unchecked(63..122) }.parse().unwrap();
        let client = atrium_api::client::AtpServiceClient::new(
            atrium_xrpc_client::reqwest::ReqwestClient::new(host.clone()),
        );
        let blob = Compat::new(client
            .service
            .com
            .atproto
            .sync
            .get_blob(atrium_api::com::atproto::sync::get_blob::ParametersData { cid, did }.into()))
            .await;
        Ok(VecReader::new(
            blob.map_err(|_| AssetReaderError::NotFound(path.into()))?,
        ))
    }

    async fn read_directory<'a>(
        &'a self,
        _: &'a Path,
    ) -> Result<Box<bevy::asset::io::PathStream>, AssetReaderError> {
        todo!()
    }

    async fn is_directory<'a>(&'a self, _: &'a Path) -> Result<bool, AssetReaderError> {
        todo!()
    }

    async fn read_meta<'a>(&'a self, _: &'a Path) -> Result<VecReader, AssetReaderError> {
        todo!()
    }
}
