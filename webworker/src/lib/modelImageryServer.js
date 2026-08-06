import GetMapWorker from "./getMap.worker.js?worker"
import PromiseWorker from 'promise-worker';

export default class ModelImageryServer {
    constructor() {
        this.worker = new PromiseWorker(new GetMapWorker());
    }
    requestImage(url) {
        return this.worker.postMessage(url).then(res => {
            return res.imageData;
        })
    }
    destroy() {
        this.worker.terminate()
        this.worker = null
    }
}