export class RingBuffer {
  private buffer: Float32Array;
  private size: number;
  private head: number = 0;
  private tail: number = 0;
  private full: boolean = false;

  constructor(size: number) {
    this.size = size;
    this.buffer = new Float32Array(size);
  }

  push(value: number) {
    this.buffer[this.head] = value;
    this.head = (this.head + 1) % this.size;
    if (this.full) {
      this.tail = (this.tail + 1) % this.size;
    }
    if (this.head === this.tail && !this.full) {
      this.full = true;
    }
  }

  pushMany(values: number[]) {
    for (const v of values) {
      this.push(v);
    }
  }

  get(index: number): number {
    if (index < 0 || index >= this.length) return 0;
    return this.buffer[(this.tail + index) % this.size];
  }

  get length(): number {
    if (this.full) return this.size;
    return (this.head - this.tail + this.size) % this.size;
  }

  clear() {
    this.head = 0;
    this.tail = 0;
    this.full = false;
  }

  toArray(): Float32Array {
    const result = new Float32Array(this.length);
    for (let i = 0; i < this.length; i++) {
      result[i] = this.get(i);
    }
    return result;
  }
}
