export function getRandomInt(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min
}

export function getRandomU32() {
  return Math.floor(Math.random() * 0xffffffff)
}
