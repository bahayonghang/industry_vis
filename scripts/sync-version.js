#!/usr/bin/env node
/**
 * 版本号同步脚本
 * 
 * 从 package.json 读取版本号，同步到：
 * - src-tauri/Cargo.toml
 * - src-tauri/tauri.conf.json
 * - src/version.ts (前端版本号)
 * 
 * 用法: node scripts/sync-version.js
 */

import { readFileSync, writeFileSync } from 'fs'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const rootDir = join(__dirname, '..')

// 读取 package.json 版本
const packageJson = JSON.parse(readFileSync(join(rootDir, 'package.json'), 'utf-8'))
const version = packageJson.version

console.log(`📦 同步版本号: ${version}`)

// 同步 Cargo.toml
const cargoPath = join(rootDir, 'src-tauri', 'Cargo.toml')
let cargoContent = readFileSync(cargoPath, 'utf-8')
const cargoVersionRegex = /^version\s*=\s*"[^"]*"/m
if (cargoVersionRegex.test(cargoContent)) {
  const oldVersion = cargoContent.match(cargoVersionRegex)[0]
  cargoContent = cargoContent.replace(cargoVersionRegex, `version = "${version}"`)
  writeFileSync(cargoPath, cargoContent)
  console.log(`  ✅ Cargo.toml: ${oldVersion} → version = "${version}"`)
} else {
  console.log(`  ⚠️ Cargo.toml: 未找到版本号字段`)
}

// 同步 tauri.conf.json
const tauriConfPath = join(rootDir, 'src-tauri', 'tauri.conf.json')
const tauriConf = JSON.parse(readFileSync(tauriConfPath, 'utf-8'))
const oldTauriVersion = tauriConf.version
tauriConf.version = version
writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n')
console.log(`  ✅ tauri.conf.json: "${oldTauriVersion}" → "${version}"`)

// 同步前端版本号文件
const versionTsPath = join(rootDir, 'src', 'version.ts')
const versionTsContent = `// 此文件由 scripts/sync-version.js 自动生成，请勿手动修改
export const APP_VERSION = '${version}'
`
writeFileSync(versionTsPath, versionTsContent)
console.log(`  ✅ src/version.ts: APP_VERSION = '${version}'`)

console.log(`\n🎉 版本同步完成!`)
