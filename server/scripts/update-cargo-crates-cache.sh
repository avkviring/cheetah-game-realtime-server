# Обновление cargo crates cache для offline сборки на CI
cd ../
cargo fetcher --url file:///`pwd`/../.cache/crates/ --include-index mirror