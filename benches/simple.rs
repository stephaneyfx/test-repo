use brightness::Brightness;
use criterion::{criterion_group, criterion_main, Criterion};
use futures::TryStreamExt;
use std::net::{Ipv4Addr, UdpSocket};
use tokio::runtime::Runtime;

const PATH: &str = "/sys/class/backlight/amdgpu_bl0/brightness";

async fn brightness_async() {
    brightness::brightness_devices()
        .try_for_each(|dev| async move {
            let _ = dev.device_name().await?;
            let _ = dev.get().await?;
            Ok(())
        })
        .await
        .unwrap();
}

async fn brightness_tokio() {
    let mut entries = tokio::fs::read_dir("/sys/class/backlight").await.unwrap();
    while let Some(entry) = entries.next_entry().await.unwrap() {
        let path = entry.path();
        let _ = tokio::fs::read_to_string(path.join("actual_brightness"))
            .await
            .unwrap();
    }
}

async fn std_fs_in_async() -> String {
    std::fs::read_to_string(PATH).unwrap()
}

async fn tokio_fs() -> String {
    tokio::fs::read_to_string(PATH).await.unwrap()
}

async fn tokio_spawn_nothing() {
    tokio::task::spawn_blocking(|| ()).await.unwrap()
}

fn alloc() -> Vec<u8> {
    vec![33; 1_000_000]
}

fn new_socket() -> UdpSocket {
    UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap()
}

fn make_runtime() -> Runtime {
    Runtime::new().unwrap()
}

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple");
    group.bench_function("alloc", |b| b.iter(|| alloc()));
    group.bench_function("new_socket", |b| b.iter(|| new_socket()));
    group.bench_function("brightness_async", |b| {
        b.to_async(make_runtime()).iter(|| brightness_async())
    });
    group.bench_function("brightness_tokio", |b| {
        b.to_async(make_runtime()).iter(|| brightness_tokio())
    });
    group.bench_function("std_fs_in_async", |b| {
        b.to_async(make_runtime()).iter(|| std_fs_in_async())
    });
    group.bench_function("tokio_fs", |b| {
        b.to_async(make_runtime()).iter(|| tokio_fs())
    });
    group.bench_function("tokio_spawn_nothing", |b| {
        b.to_async(make_runtime()).iter(|| tokio_spawn_nothing())
    });
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
