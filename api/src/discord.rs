// use std::collections::HashMap;
// use http::StatusCode;
// use twilight_http::Client;
// use twilight_model::guild::Guild;
// use twilight_model::id::Id;
// use twilight_model::id::marker::GuildMarker;
//
//
// struct TimedValue<V> {
//     value: Option<V>,
//     fetched_at: u64,
// }
//
// struct GuildCache <'a> {
//     cache: HashMap<Id<GuildMarker>, TimedValue<Guild>>,
//     client: &'a Client
// }
//
// trait CacheFetch {
//     async fn fetch(&self, key: Id<GuildMarker>) -> Result<Option<Guild>, StatusCode>;
// }
//
// impl CacheFetch for GuildCache<'_> {
//     async fn fetch(&self, key: Id<GuildMarker>) -> Result<Option<Guild>, StatusCode>{
//         Ok(Some(self.client.guild(key).await.unwrap().model().await.unwrap()))
//     }
// }
// // }
// //
// // struct DscCache<'a, K, V, F, Fut>
// // where
// //     K: std::hash::Hash + Eq + Clone,
// //     F: Fn(&Client, K) -> Fut,
// //     Fut: Future<Output = Result<Option<V>, StatusCode>> + ?Sized,
// // {
// //     cache: HashMap<K, TimedValue<V>>,
// //     client: &'a Client,
// //     fetcher: F,
// // }
// //
// // impl<'a, K, V, F, Fut> DscCache<'a, K, V, F, Fut>
// // where
// //     K: std::hash::Hash + Eq + Clone,
// //     F: Fn(&Client, K) -> Fut,
// //     Fut: Future<Output = Result<Option<V>, StatusCode>>,
// // {
// //     pub fn new(client: &'a Client, fetcher: F) -> Self {
// //         Self {
// //             cache: HashMap::new(),
// //             client,
// //             fetcher
// //         }
// //     }
// //
// //     pub async fn get_or_fetch(&mut self, key: K) -> Result<&Option<V>, StatusCode> {
// //         if let Some(timed_value) = self.cache.get(&key) {
// //             if timed_value.fetched_at + 300 > SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() {
// //                 // Cache is valid for 5 minutes
// //                 return Ok(&timed_value.value);
// //             } else {
// //                 // Cache is stale, we should fetch a new value
// //                 self.cache.remove(&key);
// //             }
// //         }
// //         let value = (self.fetcher)(self.client, key.clone()).await?;
// //         let timed_value = TimedValue {
// //             value,
// //             fetched_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
// //         };
// //         self.cache.insert(key.clone(), timed_value);
// //         Ok(&self.cache.get(&key).unwrap().value)
// //     }
// // }
// //
// //
// // struct DscCacheGroup<'a> {
// //     guild: DscCache<'a, Id<GuildMarker>, Guild, dyn Fn(&Client, Id<GuildMarker>) -> dyn Future<Output=Result<Option<Guild>, StatusCode>>, dyn Future<Output = Result<Option<Guild>, StatusCode>>>
// // }
// //
// //
// // async fn fetch_guild(client: &Client, guild_id: Id<GuildMarker>) -> Result<Option<Guild>, StatusCode> {
// //     match client.guild(guild_id).await {
// //         Ok(response) => {
// //             Ok(Some(response.model().await.map_err(ise)?))
// //         },
// //         Err(e) => {
// //             match e.kind() {
// //                 twilight_http::error::ErrorType::Response{body, error,status} => {
// //                     match status {
// //                         &twilight_http::response::StatusCode::NOT_FOUND => Ok(None),
// //                         _ => Err(ise(e))
// //                     }
// //                 },
// //                 _ => Err(ise(e))
// //             }
// //         }
// //
// //     }
// // }
// //
// // impl<'a> DscCacheGroup<'a> {
// //     pub fn new(client: &'a Client) -> Self {
// //         Self {
// //             guild: DscCache::new(client, |client, guild_id| {
// //                 Box::pin(async move {
// //                     match client.guild(guild_id).await {
// //                         Ok(response) => {
// //                             Ok(Some(response.model().await.map_err(ise)?))
// //                         },
// //                         Err(e) => {
// //                             match e.kind() {
// //                                 twilight_http::error::ErrorType::Response { body, error, status } => {
// //                                     match status {
// //                                         &twilight_http::response::StatusCode::NOT_FOUND => Ok(None),
// //                                         _ => Err(ise(e))
// //                                     }
// //                                 },
// //                                 _ => Err(ise(e))
// //                             }
// //                         }
// //                     }
// //                 })
// //             }),
// //         }
// //     }
// // }
//
// //
// // struct KbDscClient {
// //     client: twilight_http::Client,
// //     cache: DscCacheGroup,
// // }
// //
// // impl KbDscClient {
// //     pub fn new(token: String) -> Self {
// //         Self {
// //             client: twilight_http::Client::new(token),
// //             cache: DscCacheGroup::new(),
// //         }
// //     }
// //
// //     pub async fn fetch_guild(&mut self, guild_id: Id<GuildMarker>) -> Result<&Option<Guild>, StatusCode> {
// //         if self.cache.guild.contains_key(&guild_id) {
// //             return Ok(self.cache.guild.get(&guild_id).unwrap());
// //         }
// //
// //         match self.client.guild(guild_id).await {
// //             Ok(response) => {
// //                 match response.model().await {
// //                     Ok(guild) => {
// //                         self.cache.guild.insert(guild_id, Some(guild));
// //                         Ok(self.cache.guild.get(&guild_id).unwrap())
// //                     }
// //                     Err(e) => Err(ise(e))
// //                 }
// //             }
// //             Err(e) => Err(ise(e))
// //         }
// //     }
// // }