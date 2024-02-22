import { writeFile } from "fs"

export const saveJsonToFile = (filename: string, jsonContent: string) => {
  writeFile(filename, jsonContent, "utf8", (err) => {
    if (err) {
      console.error("An error occurred:", err)
      return
    }
    console.log("JSON saved to", filename)
  })
}
