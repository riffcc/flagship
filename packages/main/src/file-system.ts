import {dialog, ipcMain, type IpcMainInvokeEvent} from 'electron';

const chooseDirectory = async (_event: IpcMainInvokeEvent): Promise<string | undefined> => {
  return (
    await dialog.showOpenDialog({
      properties: ['openDirectory', 'promptToCreate'],
    })
  ).filePaths[0];
};

export const connectFileSystem = () => {
  ipcMain.handle('chooseDirectory', chooseDirectory);
};
