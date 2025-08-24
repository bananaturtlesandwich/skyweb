use super::*;
use bevy::asset::io::{AssetReader, AssetReaderError, VecReader};
use std::path::Path;

pub struct Stuff;

impl Plugin for Stuff {
    fn build(&self, app: &mut App) {
        app.register_asset_source(
            "https",
            bevy::asset::io::AssetSource::build().with_reader(|| Box::new(AvatarReader)),
        )
        .add_systems(Update, process.after(bevy::asset::AssetEvents));
    }
}

static CLIENT: std::sync::LazyLock<reqwest::Client> =
    std::sync::LazyLock::new(reqwest::Client::new);

struct AvatarReader;

impl AssetReader for AvatarReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<VecReader, AssetReaderError> {
        let Some(url) = path.to_str() else {
            return Err(AssetReaderError::NotFound(path.into()));
        };
        // the avatar url in the profile view data links to the cdn
        // https://cdn.bsky.app/img/avatar/plain/did:plc:vt545bncnkhhuhceflma2vxv/bafkreibkqsetx47ccocfse7mxdujmxgm57si5a5leerifnlwhmka2awr3u@jpeg
        // however requests to the cdn are blocked by the CORS policy on wasm
        // so we have to get the pds host and request the avatar from there
        // https://blusher.us-east.host.bsky.network/xrpc/com.atproto.sync.getBlob?did=did:plc:vt545bncnkhhuhceflma2vxv&cid=bafkreibkqsetx47ccocfse7mxdujmxgm57si5a5leerifnlwhmka2awr3u
        #[cfg(target_family = "wasm")]
        if url.starts_with("cdn.bsky.app/img/avatar") {
            let did = unsafe { url.get_unchecked(30..62) };
            let req = Compat::new(
                CLIENT
                    .get(String::from("https://plc.directory/") + did)
                    .send(),
            )
            .await
            .map_err(|_| AssetReaderError::NotFound(path.into()))?;
            let text = Compat::new(req.text())
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
            let blob = Compat::new(client.service.com.atproto.sync.get_blob(
                atrium_api::com::atproto::sync::get_blob::ParametersData { cid, did }.into(),
            ))
            .await;
            return Ok(VecReader::new(
                blob.map_err(|_| AssetReaderError::NotFound(path.into()))?,
            ));
        }
        let req = Compat::new(CLIENT.get(format!("https://{url}")).send())
            .await
            .map_err(|_| AssetReaderError::NotFound(path.into()))?;
        let blob = Compat::new(req.bytes())
            .await
            .map_err(|_| AssetReaderError::NotFound(path.into()))?;
        Ok(VecReader::new(blob.to_vec()))
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

fn process(mut events: EventReader<AssetEvent<Image>>, mut images: ResMut<Assets<Image>>) {
    for event in events.read() {
        let AssetEvent::LoadedWithDependencies { id } = event else {
            continue;
        };
        let Some(image) = images.get_mut(*id) else {
            continue;
        };
        match std::mem::take(image).try_into_dynamic() {
            Ok(dynamic) => {
                *image = Image::from_dynamic(
                    dynamic.thumbnail(64, 64),
                    image.texture_descriptor.format.is_srgb(),
                    bevy::asset::RenderAssetUsages::default(),
                )
            }
            Err(e) => bevy::log::error!("{e}"),
        }
    }
}
