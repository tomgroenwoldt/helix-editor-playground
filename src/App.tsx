import { Box, Button, IconButton } from "@chakra-ui/react";
import { useState } from "react";
import "./App.css";
import Editor from "./Editor";

export enum Version {
  Release,
  Master,
}

function App() {
  const [version, setVersion] = useState<Version | undefined>(undefined);
  if (version === undefined) {
    return (
      <div id="main">
        <Box id={"choice"}>
          <Box
            id={"choice-value"}
            onClick={() => {
              console.log("test");
              setVersion(Version.Release);
            }}
          >
            <img id={"logo"} src={process.env.PUBLIC_URL + "/logo.png"} />
            <code>{"release 23.05"}</code>
          </Box>
          <Box id={"choice-value"} onClick={() => setVersion(Version.Master)}>
            <img
              id={"logo"}
              src={process.env.PUBLIC_URL + "/github-mark-white.png"}
            />
            <code>{"master 8d39a81"}</code>
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
