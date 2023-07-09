import { Box, Spinner } from "@chakra-ui/react";
import { useEffect, useState } from "react";
import "./App.css";
import Editor from "./Editor";

export enum Version {
  Release,
  Master,
}

interface BackendVersions {
  /** Latest helix release version */
  release: string;
  /** Latest helix master version */
  master: string;
}

function App() {
  const [version, setVersion] = useState<Version | undefined>(undefined);
  const [backendVersions, setBackendVersions] = useState<
    BackendVersions | undefined
  >(undefined);

  // Fetch versions of helix instances served by backend
  useEffect(() => {
    fetch("https://tomgroenwoldt.de/versions").then((res) =>
      res.json().then(setBackendVersions)
    );
  }, []);

  if (version === undefined) {
    return (
      <div id="main">
        <Box id={"center"}>
          <Box id={"header"}>{"choose between latest release or master"}</Box>
          <Box id={"choice"}>
            <Box
              id={"choice-value"}
              onClick={() => {
                if (backendVersions) {
                  setVersion(Version.Release);
                }
              }}
            >
              <img id={"logo"} src={process.env.PUBLIC_URL + "/logo.png"} />
              {backendVersions === undefined ? (
                <Spinner id={"spinner"} thickness={"3px"} />
              ) : (
                <code id={"spinner"}>{backendVersions.release}</code>
              )}
            </Box>
            <Box
              id={"choice-value"}
              onClick={() => {
                if (backendVersions) {
                  setVersion(Version.Master);
                }
              }}
            >
              <img
                id={"logo"}
                src={process.env.PUBLIC_URL + "/github-mark-white.png"}
              />
              {backendVersions === undefined ? (
                <Spinner id={"spinner"} thickness={"3px"} />
              ) : (
                <code id={"spinner"}>{backendVersions.master}</code>
              )}
            </Box>
          </Box>
          <Box id={"footer"}>{"updated daily"}</Box>
          <Box
            id={"button"}
            onClick={() => {
              window.location.href =
                "https://github.com/tomgroenwoldt/helix-editor-playground";
            }}
          >
            {"View source"}
          </Box>
        </Box>
      </div>
    );
  }
  return (
    <div id="main">
      <Editor version={version} />
    </div>
  );
}

export default App;
