import { ITerminalOptions, Terminal } from "xterm";
import "xterm/css/xterm.css";
import "./Editor.css";
import { Box } from "@chakra-ui/react";
import { WebglAddon } from "xterm-addon-webgl";
import { AttachAddon } from "./AttachAddon";

const Watcher: React.FC = () => {
  // Create xtermjs terminal.
  const terminalOptions: ITerminalOptions = {
    theme: { background: "#282a36" },
    fontSize: 20,
    macOptionClickForcesSelection: true,
    scrollback: 0,
  };
  const term = new Terminal(terminalOptions);

  const ws = new WebSocket("ws://127.0.0.1:8080/watch/bebenulautus-dubonules");
  ws.addEventListener("open", (_event) => {
    // Load addons.
    const attachAddon = new AttachAddon(ws);
    term.loadAddon(attachAddon);
    term.loadAddon(new WebglAddon());

    term.open(document.getElementById("editor")!);
    term.focus();
  });

  return <Box id={"editor"} h="full"></Box>;
};

export default Watcher;
