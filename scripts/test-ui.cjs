'use strict';
// Discover files in Node, not in the shell (including Windows PowerShell).
const {readdirSync}=require('node:fs');
const {join,resolve}=require('node:path');
const {spawnSync}=require('node:child_process');
const root=resolve(__dirname,'..');
function discover(directory){
 return readdirSync(directory,{withFileTypes:true}).flatMap(entry=>{
  const path=join(directory,entry.name);
  return entry.isDirectory()?discover(path):entry.isFile()&&entry.name.endsWith('.test.cjs')?[path]:[];
 }).sort();
}
const files=discover(join(root,'tests'));
if(!files.length)throw Error('No interface test files found');
if(process.argv.includes('--list'))console.log(files.join('\n'));
else {
 const result=spawnSync(process.execPath,['--test',...files],{cwd:root,stdio:'inherit'});
 if(result.error)throw result.error;
 process.exitCode=result.status??1;
}
