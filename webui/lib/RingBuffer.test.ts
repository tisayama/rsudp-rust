import { RingBuffer } from './RingBuffer';

describe('RingBuffer', () => {
  it('should initialize with correct size', () => {
    const rb = new RingBuffer(10);
    expect(rb.length).toBe(0);
  });

  it('should push values and report correct length', () => {
    const rb = new RingBuffer(5);
    rb.push(1);
    rb.push(2);
    expect(rb.length).toBe(2);
    expect(rb.get(0)).toBe(1);
    expect(rb.get(1)).toBe(2);
  });

  it('should wrap around when full', () => {
    const rb = new RingBuffer(3);
    rb.push(1);
    rb.push(2);
    rb.push(3);
    expect(rb.length).toBe(3);
    expect(rb.get(0)).toBe(1);
    expect(rb.get(2)).toBe(3);

    rb.push(4); // Overwrites 1
    expect(rb.length).toBe(3);
    expect(rb.get(0)).toBe(2);
    expect(rb.get(1)).toBe(3);
    expect(rb.get(2)).toBe(4);
  });

  it('should handle pushMany', () => {
    const rb = new RingBuffer(5);
    rb.pushMany([1, 2, 3]);
    expect(rb.length).toBe(3);
    expect(rb.get(2)).toBe(3);

    rb.pushMany([4, 5, 6]); // Wraps
    expect(rb.length).toBe(5);
    expect(rb.get(0)).toBe(2);
    expect(rb.get(4)).toBe(6);
  });

  it('should clear correctly', () => {
    const rb = new RingBuffer(5);
    rb.pushMany([1, 2, 3]);
    rb.clear();
    expect(rb.length).toBe(0);
    expect(rb.get(0)).toBe(0); // Returns default for out of bounds
  });

  it('should convert to array correctly', () => {
    const rb = new RingBuffer(3);
    rb.pushMany([1, 2, 3, 4]); // Wraps: [2, 3, 4]
    const arr = rb.toArray();
    expect(arr).toEqual(new Float32Array([2, 3, 4]));
  });
});
