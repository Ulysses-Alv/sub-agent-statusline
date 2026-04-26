export interface KeyHandlerActions {
  onMoveUp: () => void;
  onMoveDown: () => void;
  onDelete: () => void;
  onConfirm: () => void;
  onCancel: () => void;
  onQuit: () => void;
}

export function handleKeyPress(actions: KeyHandlerActions): (key: string) => void {
  return (key: string): void => {
    switch (key) {
      case "w":
      case "up":
      case "k":
        actions.onMoveUp();
        break;
      case "s":
      case "down":
      case "j":
        actions.onMoveDown();
        break;
      case "d":
        actions.onDelete();
        break;
      case "y":
      case "Y":
        actions.onConfirm();
        break;
      case "n":
      case "N":
      case "escape":
      case "c_c":
        actions.onCancel();
        break;
      case "q":
      case "Q":
      case "c_c":
        actions.onQuit();
        break;
    }
  };
}