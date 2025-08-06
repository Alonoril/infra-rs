use base_infra::tools::retry::Retry;

async fn fetch_data() -> Result<String, reqwest::Error> {
	// 这里是你的网络请求逻辑
	reqwest::get("https://example.com").await?.text().await
}

#[tokio::main]
async fn main() {
	let retry_future = Retry::run(None, || fetch_data()); // 自定义重试5次
	match retry_future.await {
		Ok(data) => println!("请求成功: {}", data),
		Err(e) => eprintln!("请求失败: {}", e),
	}
}
