MODE=$1

err="$(kubectl create namespace bots 2>&1)"
if ! (grep -q "AlreadyExists" <<< "$err"); then
    echo $err
    exit 1
fi

if [ "$1" = "prod" ]; then
    BIN="target/x86_64-unknown-linux-musl/release/elgua"

    cargo build --release --target=x86_64-unknown-linux-musl
else
    BIN="target/x86_64-unknown-linux-musl/debug/elgua"

    cargo build --target=x86_64-unknown-linux-musl
fi

if [ $? -ne 0 ]; then
    exit 1
fi

chmod +x $BIN

docker build \
    --platform linux/amd64 \
    --build-arg BINARY_FILE="$BIN" \
    --build-arg CFG_FILE="cfg.json" \
    --tag "192.168.1.21:32000/elgua:latest" \
    --load .

if [ $? -ne 0 ]; then
    exit 1
fi

docker push "192.168.1.21:32000/elgua:latest" &&
# kubectl get "deployment/elgua" &&
kubectl apply -f k8s/storage -f k8s/deployment.yaml &&
kubectl rollout restart "deployment/elgua" -n bots
