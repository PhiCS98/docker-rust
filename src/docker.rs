use std::{fs::File, io::Cursor};

use anyhow::{Ok, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct Auth {
    token: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Layer {
    digest: String,
}

#[derive(Deserialize, Debug)]
pub struct Manifest {
    pub layers: Vec<Layer>,
}
pub struct Docker {}

impl Docker {
    pub fn get_manifest(image_name: &str, image_reference: &str) -> Result<Manifest> {
        const DOCKER_REGISTRY_URL: &str = "https://registry.hub.docker.com";
        let url =
            format!("{DOCKER_REGISTRY_URL}/v2/library/{image_name}/manifests/{image_reference}");
        let token = get_auth_token(image_name)?;
        let client = reqwest::blocking::Client::new();
        let manifest = client
            .get(url)
            .header(
                reqwest::header::AUTHORIZATION,
                "Bearer ".to_owned() + &token,
            )
            .header(
                reqwest::header::ACCEPT,
                "application/vnd.docker.distribution.manifest.v2+json",
            )
            .send()?
            .json::<Manifest>()?;
        Ok(manifest)
    }

    pub fn get_layer(image_name: &str, layer: &Layer) -> Result<File> {
        const DOCKER_REGISTRY_URL: &str = "https://registry.hub.docker.com";
        let url = format!(
            "{DOCKER_REGISTRY_URL}/v2/library/{image_name}/blobs/{}",
            layer.digest
        );
        let token = get_auth_token(image_name)?;
        let client = reqwest::blocking::Client::new();
        let res = client
            .get(url)
            .header(
                reqwest::header::AUTHORIZATION,
                "Bearer ".to_owned() + &token,
            )
            .send()?;
        let mut content = Cursor::new(res.bytes()?);
        let mut file = tempfile::tempfile()?;
        std::io::copy(&mut content, &mut file)?;
        Ok(file)
    }
}

fn get_auth_token(image: &str) -> Result<String> {
    const AUTH_DOCKER_REGISTRY_URL: &str = "https://auth.docker.io/token";
    let params = [
        ("service", "registry.docker.io"),
        ("scope", &format!("repository:library/{image}:pull")),
    ];
    let url = reqwest::Url::parse_with_params(AUTH_DOCKER_REGISTRY_URL, &params)?;
    let res = reqwest::blocking::get(url)?.json::<Auth>()?;
    Ok(res.token)
}
