import { ITerminalOptions, Terminal } from "xterm";
import { FitAddon } from "xterm-addon-fit";
import "xterm/css/xterm.css";
import "./Editor.css";
import { Box } from "@chakra-ui/react";
import { WebglAddon } from "xterm-addon-webgl";
import { AttachAddon } from "./AttachAddon";

const Editor: React.FC = () => {
  // Create xtermjs terminal.
  const terminalOptions: ITerminalOptions = {
    theme: { background: "#282a36" },
    fontSize: 20,
    macOptionClickForcesSelection: true,
    scrollback: 0,
  };
  const term = new Terminal(terminalOptions);

  const ws = new WebSocket("wss://tomgroenwoldt.de/helix");
  ws.addEventListener("open", (_event) => {
    // Load addons.
    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    const attachAddon = new AttachAddon(ws);
    term.loadAddon(attachAddon);
    term.loadAddon(new WebglAddon());

    term.open(document.getElementById("editor")!);
    term.focus();

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

    // Fit the terminal on first render.
    fitAddon.fit();
  });

  console.log("rendering...");

  return <Box id={"editor"} h="full"></Box>;
};

export default Editor;
