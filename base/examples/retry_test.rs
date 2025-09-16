use base_infra::tools::retry::Retry;

async fn fetch_data() -> Result<String, reqwest::Error> {
	// Your network request logic here
	reqwest::get("https://example.com").await?.text().await
}

#[tokio::main]
async fn main() {
	let retry_future = Retry::run(None, || fetch_data()); // Customize retries to 5
	match retry_future.await {
		Ok(data) => println!("Request succeeded: {}", data),
		Err(e) => eprintln!("Request failed: {}", e),
	}
}
