const COLUMNS = 80;
const MINIMUM_ROWS = 24;

const charTable = {};

const u8_to_char = u8 => {
  if (charTable[u8] === undefined) {
    charTable[u8] = String.fromCharCode(u8);
  }
  return charTable[u8];
};

const u8s_to_strings = u8s => {
  let s = '';
  for (const u8 of u8s) {
    s += u8_to_char(u8);
  }
  return s;
};

export default class App {
  constructor(riscv, terminal, options = {}) {
    this.riscv = riscv;
    this.terminal = terminal;
    this.debugModeEnabled = options.debugModeEnabled !== undefined ? options.debugModeEnabled : false;
    this.runCyclesNum = options.runCyclesNum !== undefined ? options.runCyclesNum : 0x10000;
    this.inDebugMode = false;
    this.inputs = [];
    this.lastCommandStrings = '';
    this._setupInputEventHandlers();
  }

  _setupInputEventHandlers() {
    this.terminal.onKey(event => {
      if (this.inDebugMode) {
        this._handleKeyInputInDebugMode(event.key, event.domEvent.keyCode);
      } else {
        this._handleKeyInput(event.key, event.domEvent.keyCode);
      }
    });
    // I don't know why but terminal.onKey doesn't catch
    // space key so handling in document keydown event listener.
    document.addEventListener('keydown', event => {
      if (event.keyCode === 32) {
        if (this.inDebugMode) {
          this._handleKeyInputInDebugMode(' ', 32);
        } else {
          this._handleKeyInput(' ', 32);
        }
      }
      event.preventDefault();
    });
  }

  _handleKeyInput(key, keyCode) {
    if (this.debugModeEnabled && key.charCodeAt(0) === 1) { // Ctrl-A
      this.enterDebugMode();
      return;
    }

    const inputs = this.inputs;

    // xterm.js doesn't handle function keys so
    // handling by myself here
    switch (keyCode) {
      case 32: // Space
        inputs.push(32);
        break;
      case 33: // Page up
        inputs.push(27, 91, 53, 126);
        break;
      case 34: // Page down
        inputs.push(27, 91, 54, 126);
        break;
      case 35: // End
        inputs.push(27, 91, 52, 126);
        break;
      case 36: // Home
        inputs.push(27, 91, 49, 126);
        break;
      case 37: // Arrow Left
        inputs.push(27, 91, 68);
        break;
      case 38: // Arrow Up
        inputs.push(27, 91, 65);
        break;
      case 39: // Arrow Right
        inputs.push(27, 91, 67);
        break;
      case 40: // Arrow Down
        inputs.push(27, 91, 66);
        break;
      case 45: // Insert
        inputs.push(27, 91, 50, 126);
        break;
      case 46: // Delete
        inputs.push(127);
        break;
      case 112: // F1
        inputs.push(27, 79, 80);
        break;
      case 113: // F2
        inputs.push(27, 79, 81);
        break;
      case 114: // F3
        inputs.push(27, 79, 82);
        break;
      case 115: // F4
        inputs.push(27, 79, 83);
        break;
      case 116: // F5
        inputs.push(27, 91, 49, 53, 126);
        break;
      case 117: // F6
        inputs.push(27, 91, 49, 55, 126);
        break;
      case 118: // F7
        inputs.push(27, 91, 49, 6, 126);
        break;
      case 119: // F8
        inputs.push(27, 91, 49, 57, 126);
        break;
      case 120: // F9
        inputs.push(27, 91, 50, 48, 126);
        break;
      case 121: // F10
        inputs.push(27, 91, 50, 49, 126);
        break;
      case 122: // F11
        inputs.push(27, 91, 50, 50, 126);
        break;
      case 123: // F12
        inputs.push(27, 91, 50, 51, 126);
        break;
      default:
        inputs.push(key.charCodeAt(0));
        break;
    }
  }

  _handleKeyInputInDebugMode(key, keyCode) {
    switch(keyCode) {
      case 8: // backspace
        // Do not delete the prompt
        if (this.terminal._core.buffer.x > 2) {
          this.terminal.write('\b \b');
        }
        break;
      case 13: // new line
        const lines = this.terminal._core.buffer.lines;
        // Is there easier way to get last line?
        const y = this.terminal._core.buffer.y < this.terminal.rows - 1
          ? this.terminal._core.buffer.y : lines.length - 1;
        const line = lines.get(y);
        const length = line.getTrimmedLength();

        let commandStrings = '';
        for (let i = 2; i < length; i++) {
          commandStrings += line.getString(i);
        }
        if (commandStrings.trim() === '') {
          commandStrings = this.lastCommandStrings;
        }
        const command = this._parseCommand(commandStrings);
        this.lastCommandStrings = commandStrings;
        if (!this._runCommand(command)) {
          this.terminal.writeln('');
          this.terminal.write('Unknown command.');
        }
        if (this.inDebugMode) {
          this.prompt();
        }
        break;
      default:
        this.terminal.write(key);
        break;
    }
  }

  _parseCommand(s) {
    return s.trim().split(/\s+/);
  }

  _runCommand(command) {
    if (command.length === 0) {
      return false;
    }
    switch(command[0].toLowerCase()) {
      case '':
        // Do nothing
        return command.length === 1;
	    break;
      // UGH...
      case 'c':
      case 'co':
      case 'con':
      case 'cont':
      case 'conti':
      case 'contin':
      case 'continu':
      case 'continue':
        if (command.length === 1) {
          this.continue();
          return true;
        } else {
          return false;
        }
        break;
      case 'h':
      case 'he':
      case 'hel':
      case 'help':
        if (command.length === 1) {
          this.terminal.writeln('');
          this.displayHelp();
          return true;
        } else {
          return false;
        }
        break;
      case 's':
      case 'st':
      case 'ste':
      case 'step':
        switch (command.length) {
          case 1:
            this.step(1);
            return true;
          case 2:
            const num = parseInt(command[1]);
            if (isNaN(num)) {
              return false;
            }
            this.step(num);
            return true;
          default:
            return false;
	    }
        break;
      default:
        return false;
    }
  }

  displayHelp() {
    this.terminal.writeln('Commands:');
    this.terminal.writeln('  continue: Continue the main program. Ctrl-A enters debug mode again.');
    this.terminal.writeln('  help: Show this message');
    this.terminal.writeln('  step [num(default=1)]: Run num step execution');
  }

  step(num) {
    this.terminal.writeln('');
    this.riscv.run_cycles(num);
    this.flush();
    this.riscv.disassemble_next_instruction();
  }

  continue() {
    const runCycles = () => {
      if (this.inDebugMode) {
        return;
      }
      setTimeout(runCycles, 0);
      this.riscv.run_cycles(this.runCyclesNum);
      this.flush();
      while (this.inputs.length > 0) {
        this.riscv.put_input(this.inputs.shift());
      }
    };

    this.inDebugMode = false;
    this.lastCommandStrings = '';
    this.terminal.writeln('');
    runCycles();
  }

  flush() {
    const outputBytes = [];
    while (true) {
      const data = this.riscv.get_output();
      if (data !== 0) {
        outputBytes.push(data);
      } else {
        break;
      }
    }
    if (outputBytes.length > 0) {
      this.terminal.write(u8s_to_strings(outputBytes));
    }
  }

  enterDebugMode() {
    this.inDebugMode = true;
    this.flush();
    this.terminal.writeln('');
    this.riscv.disassemble_next_instruction();
    this.prompt();
  }

  prompt() {
    this.flush();
    this.terminal.write('\r\n% ');
  }
}
