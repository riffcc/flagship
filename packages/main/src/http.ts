import axios, {type AxiosRequestConfig, type AxiosResponse} from 'axios';
import {ipcMain, type IpcMainInvokeEvent} from 'electron';

const httpRequest = async (
  _event: IpcMainInvokeEvent,
  args: {url: string; config?: AxiosRequestConfig},
): Promise<AxiosResponse['data']> => {
  return (await axios.get(args.url, args.config)).data;
};

export const connectHttp = () => {
  ipcMain.handle('httpRequest', httpRequest);
};
