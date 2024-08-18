# Helix editor playground

A small frontend running a [xtermjs](https://xtermjs.org/) terminal attached
to a websocket server executing [helix](https://helix-editor.com) inside a
sandboxed container process.

## How to use

Pull this repository:

```bash
git clone git@github.com:tomgroenwoldt/helix-editor-playground.git
cd helix-editor-playground
```

Start the frontend:
```bash
cd frontend
npm install
npm run start
```

For the backend you can use the pre-built docker image of the GitHub action or build the image yourself. The pre-build
image is updated every day.

### Pre-built
Pull the image:
```bash
docker pull ghcr.io/tomgroenwoldt/helix-editor-playground-backend:nightly
```
Start the container:
```bash
docker run -it --user user -p 8080:8080 ghcr.io/tomgroenwoldt/helix-editor-playground-backend:nightly
```

### Build yourself
```bash
cd backend
docker build -t helix-editor-playground-backend -f Containerfile
```
Start the container
```bash
docker run -it --user user -p 8080:8080 helix-editor-playground-backend
```

Finally, visit http://localhost:3000/helix-editor-playground.
