use anyhow::{Context, anyhow};
use std::{collections::HashMap, pin::Pin};
use tokio_stream::{Stream, StreamExt};
use tonic::{Code, Request, Response, Status, Streaming, transport::Server};
use uuid::Uuid;

use crate::mmf::{
    ChunkedMmfRunRequest, Match, Roster, StreamedMmfResponse, Ticket,
    match_making_function_service_server::{
        MatchMakingFunctionService, MatchMakingFunctionServiceServer,
    },
};

pub mod mmf {
    tonic::include_proto!("open_match2");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("open-match2-descriptor");
}

#[derive(Default)]
struct Mff;

#[tonic::async_trait]
impl MatchMakingFunctionService for Mff {
    type RunStream = Pin<Box<dyn Stream<Item = Result<StreamedMmfResponse, Status>> + Send>>;

    async fn run(
        &self,
        request: Request<Streaming<ChunkedMmfRunRequest>>,
    ) -> Result<Response<Self::RunStream>, Status> {
        let mut stream = request.into_inner();

        let mut chunks = Vec::new();
        while let Some(chunked_request) = stream.next().await {
            chunks.push(chunked_request?);
        }

        let tickets: Vec<_> = chunks
            .into_iter()
            .filter_map(|chunk| chunk.profile)
            .flat_map(|profile| profile.pools.into_values())
            .filter_map(|pool| pool.participants)
            .flat_map(|roster| roster.tickets)
            .collect();

        let matches = make_matches(tickets)
            .context("Failed to create matches")
            .map_err(|_e| {
                Status::new(
                    Code::InvalidArgument,
                    "Failed to create matches with the provided tickets",
                )
            })?;

        // Return stream of responses
        let output = tokio_stream::iter(
            matches
                .into_iter()
                .map(|m| Ok(StreamedMmfResponse { r#match: Some(m) })),
        );

        Ok(Response::new(Box::pin(output) as Self::RunStream))
    }
}

fn make_matches(tickets: Vec<Ticket>) -> anyhow::Result<Vec<Match>> {
    tickets
        .chunks_exact(4)
        .map(|chunk| {
            let player_ids: Vec<_> = chunk
                .iter()
                .filter_map(|c| c.extensions.get("player_id"))
                .collect();

            if player_ids.len() != 4 {
                return Err(anyhow!(
                    "Expected to have 4 player ids to create a match, we got {}",
                    player_ids.len()
                ));
            }

            let mut rosters = HashMap::new();

            rosters.insert(
                "players".to_owned(),
                Roster {
                    name: "players".to_owned(),
                    assignment: None,
                    tickets: tickets.clone(),
                    extensions: HashMap::default(),
                },
            );

            Ok(Match {
                id: Uuid::new_v4().to_string(),
                rosters,
                extensions: HashMap::default(),
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(mmf::FILE_DESCRIPTOR_SET)
        .build_v1alpha()?;

    let addr = "127.0.0.1:8000".parse()?;

    Server::builder()
        .add_service(MatchMakingFunctionServiceServer::new(Mff))
        .add_service(reflection_service)
        .serve(addr)
        .await?;
    Ok(())
}
