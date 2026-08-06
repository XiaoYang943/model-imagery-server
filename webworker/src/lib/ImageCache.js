export default class ImageCache {
    constructor(options) {
        this.maxCapacity = options.maxCapacity;
        this.cache = {}
        this.byteLength = 0
        this.keyList = []
        this.requestCache = {}
        this.count = 0
    }
    trim() {
        if (this.byteLength < this.maxCapacity) return
        while (this.byteLength > this.maxCapacity / 2) {
            let key = this.keyList.shift()
            if (key) {
                let item = this.cache[key]
                if (item) {
                    this.byteLength -= item.byteLength
                }
                delete this.cache[key]
            }
        }
    }
    add(key, value) {
        this.trim();
        let item = new ImageCacheItem(key, value)
        this.byteLength += item.byteLength
        this.cache[key] = item;
        this.keyList.push(key)
    }
    contains(key) {
        return this.cache.hasOwnProperty(key)
    }
    /**
     * @param {number} key makeKey生成的ID
     * @returns add函数的value参数
     */
    get(key) {
        const item = this.cache[key]
        if (!item) return undefined
        return item.data
    }
    destroy() {
        this.cache = null
        this.maxCapacity = null
    }
    async _fetchImage(url) {
        const response = await fetch(url)
        if (response.status != 200) {
            console.error(await response.json())
            throw `图片请求失败,${url}`
        }
        return response.blob()
    }
    async fetchImage(url) {
        let key = makeKey(url);
        let blob = this.get(key)
        if (!blob) {
            if (!this.requestCache[key]) {
                this.requestCache[key] = this._fetchImage(url).then(blob => {
                    this.add(key, blob);
                    delete this.requestCache[key]
                    return blob
                })
            }
            blob = await this.requestCache[key]
        }
        return createImageBitmap(blob)
    }
}
class ImageCacheItem {
    constructor(id, data) {
        // this.createAt = new Date()
        // this.queryAt = new Date()
        this.id = id
        this.data = data
    }
    get byteLength() {
        return this.data.size;
    }
}

function makeKey(keyStr) {
    var hash = 0, i, chr;
    if (keyStr.length === 0) return hash;
    for (i = 0; i < keyStr.length; i++) {
        chr = keyStr.charCodeAt(i);
        hash = ((hash << 5) - hash) + chr;
        hash |= 0; // Convert to 32bit integer
    }
    return hash;
}

export const imageCache = new ImageCache({ maxCapacity: 100 * 1024 * 1024 })//100MB