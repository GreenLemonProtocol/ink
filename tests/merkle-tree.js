class MerkleTree {
  // constructor() {}
  async init(levels, elements = [], { hashFunction, zeroElement = 0 } = {}) {
    if (elements.length > this.capacity) {
      throw new Error('Tree is full');
    }
    this.levels = levels;
    this._hashFn = hashFunction;
    this.zeroElement = zeroElement;
    this._layers = [];
    const leaves = elements.slice();
    this._layers = [leaves];
    await this._buildZeros();
    await this._buildHashes();
  }

  root() {
    return this._layers[this.levels][0] ?? this._zeros[this.levels];
  }

  async _buildHashes() {
    for (let layerIndex = 1; layerIndex <= this.levels; layerIndex++) {
      const nodes = this._layers[layerIndex - 1];
      this._layers[layerIndex] = await this._processNodes(nodes, layerIndex);
    }
  }

  async _buildZeros() {
    this._zeros = [this.zeroElement];
    for (let i = 1; i <= this.levels; i++) {
      this._zeros[i] = await this._hashFn(
        this._zeros[i - 1],
        this._zeros[i - 1]
      );
    }
  }

  async _processNodes(nodes, layerIndex) {
    const length = nodes.length;
    let currentLength = Math.ceil(length / 2);
    const currentLayer = new Array(currentLength);
    currentLength--;
    const starFrom = length - (length % 2 ^ 1);
    let j = 0;
    for (let i = starFrom; i >= 0; i -= 2) {
      if (nodes[i - 1] === undefined) break;
      const left = nodes[i - 1];
      const right =
        i === starFrom && length % 2 === 1
          ? this._zeros[layerIndex - 1]
          : nodes[i];
      currentLayer[currentLength - j] = await this._hashFn(left, right);
      j++;
    }
    return currentLayer;
  }
  /**
   * Get merkle path to a leaf
   * @param {number} index Leaf index to generate path for
   * @returns {{pathElements: Object[], pathIndex: number[]}} An object containing adjacent elements and left-right index
   */
  path(index) {
    if (isNaN(Number(index)) || index < 0 || index >= this._layers[0].length) {
      throw new Error('Index out of bounds: ' + index);
    }
    let elIndex = +index;
    const pathElements = [];
    const pathIndices = [];
    const pathPositions = [];
    for (let level = 0; level < this.levels; level++) {
      pathIndices[level] = elIndex % 2;
      const leafIndex = elIndex ^ 1;
      if (leafIndex < this._layers[level].length) {
        pathElements[level] = this._layers[level][leafIndex];
        pathPositions[level] = leafIndex;
      } else {
        pathElements[level] = this._zeros[level];
        pathPositions[level] = 0;
      }
      elIndex >>= 1;
    }
    return {
      pathElements,
      pathIndices,
      pathPositions,
      pathRoot: this.root,
    };
  }

  /**
   * Insert multiple elements into the tree.
   * @param {Array} elements Elements to insert
   */
  bulkInsert(elements) {
    if (!elements.length) {
      return;
    }
    if (this._layers[0].length + elements.length > this.capacity) {
      throw new Error('Tree is full');
    }
    // First we insert all elements except the last one
    // updating only full subtree hashes (all layers where inserted element has odd index)
    // the last element will update the full path to the root making the tree consistent again
    for (let i = 0; i < elements.length - 1; i++) {
      this._layers[0].push(elements[i]);
      let level = 0;
      let index = this._layers[0].length - 1;
      while (index % 2 === 1) {
        level++;
        index >>= 1;
        this._layers[level][index] = this._hashFn(
          this._layers[level - 1][index * 2],
          this._layers[level - 1][index * 2 + 1]
        );
      }
    }
    this.insert(elements[elements.length - 1]);
  }
  indexOf(element, comparator) {
    return BaseTree_1.BaseTree.indexOf(this._layers[0], element, 0, comparator);
  }
  proof(element) {
    const index = this.indexOf(element);
    return this.path(index);
  }
  getTreeEdge(edgeIndex) {
    const edgeElement = this._layers[0][edgeIndex];
    if (edgeElement === undefined) {
      throw new Error('Element not found');
    }
    const edgePath = this.path(edgeIndex);
    return {
      edgePath,
      edgeElement,
      edgeIndex,
      edgeElementsCount: this._layers[0].length,
    };
  }
  /**
   * 🪓
   * @param count
   */
  getTreeSlices(count = 4) {
    const length = this._layers[0].length;
    let size = Math.ceil(length / count);
    if (size % 2) size++;
    const slices = [];
    for (let i = 0; i < length; i += size) {
      const edgeLeft = i;
      const edgeRight = i + size;
      slices.push({
        edge: this.getTreeEdge(edgeLeft),
        elements: this.elements.slice(edgeLeft, edgeRight),
      });
    }
    return slices;
  }
  /**
   * Serialize entire tree state including intermediate layers into a plain object
   * Deserializing it back will not require to recompute any hashes
   * Elements are not converted to a plain type, this is responsibility of the caller
   */
  serialize() {
    return {
      levels: this.levels,
      _zeros: this._zeros,
      _layers: this._layers,
    };
  }
  /**
   * Deserialize data into a MerkleTree instance
   * Make sure to provide the same hashFunction as was used in the source tree,
   * otherwise the tree state will be invalid
   */
  static deserialize(data, hashFunction) {
    const instance = Object.assign(Object.create(this.prototype), data);
    instance._hashFn = hashFunction || simpleHash_1.default;
    instance.zeroElement = instance._zeros[0];
    return instance;
  }
  toString() {
    return JSON.stringify(this.serialize());
  }
}
module.exports = MerkleTree;
