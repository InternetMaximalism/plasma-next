import * as fs from "fs/promises"

export async function readFile(path: string): Promise<string> {
  try {
    const data = await fs.readFile(path, "utf-8")
    return data
  } catch (err) {
    console.error(err)
    return ""
  }
}
