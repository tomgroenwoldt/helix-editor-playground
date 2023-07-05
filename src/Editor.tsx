import { ITerminalOptions, Terminal } from "xterm";
import { FitAddon } from "xterm-addon-fit";
import { Box } from "@chakra-ui/react";
import { WebglAddon } from "xterm-addon-webgl";
import { AttachAddon } from "./AttachAddon";
import { Version } from "./App";
import "xterm/css/xterm.css";
import "./Editor.css";

interface EditorProps {
  /** Sets the version of the helix instance connect to. */
  version: Version;
}

const Editor: React.FC<EditorProps> = (props: EditorProps) => {
  // Create xtermjs terminal
  const terminalOptions: ITerminalOptions = {
    theme: { background: "#282a36" },
    fontSize: 20,
    macOptionClickForcesSelection: true,
    scrollback: 0,
  };
  const term = new Terminal(terminalOptions);

  // Depending on the passed in version we connect to release or master helix
  const version = props.version === Version.Release ? "release" : "master";

  // Connect to websocket serving helix
  const ws = new WebSocket("wss://tomgroenwoldt.de/helix/" + version);
  ws.addEventListener("open", (_event) => {
    // Load addons
    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    const attachAddon = new AttachAddon(ws);
    term.loadAddon(attachAddon);
    term.loadAddon(new WebglAddon());

    // Finally open the terminal with focus
    term.open(document.getElementById("editor")!);
    term.focus();

    // Resize the terminal whenever the browser window is resized
    window.addEventListener("resize", () => {
      fitAddon.fit();
    });

    const encoder = new TextEncoder();
    term.onResize((data) => {
      ws.send(encoder.encode("\x01" + JSON.stringify(data)));
    });
    term.onData((data) => {
      ws.send(encoder.encode("\x00" + data));
    });

    // Fit the terminal on first render
    fitAddon.fit();
  });

  return <Box id={"editor"} h="full"></Box>;
};

export default Editor;
