# Tile Traffic

## WMS

`{bbox}` and `{srs}` are required. Your server must be able to handle SRS of `EPSG:3857`.

```bash
cargo run  --release -- \
    --lon -104.5 --lat 40 --start-zoom 4 --end-zoom 0 \
    --wms "http://localhost:3080/?SERVICE=WMS&VERSION=1.1.1&REQUEST=GetMap&LAYERS=test1&STYLES=&SRS={srs}&BBOX={bbox}&WIDTH=512&HEIGHT=512&FORMAT=image/png"
```

## ZXY

`{z}`, `{y}` and `{x}` are required

```bash
cargo run  --release -- \
    --lon -104.9 --lat 39.7 --start-zoom 10 --end-zoom 6 --bursts 32 --sleep-ms 2 \
    --zxy "http://localhost:7800/osm.points/{z}/{x}/{y}.pbf"

# This should fail! not a valid wms
cargo run  --release -- \
    --lon -104.5 --lat 40 --start-zoom 10 --end-zoom 6 \
    --wms "http://localhost:7800/osm.points/{z}/{x}/{y}.pbf"

```
