const fs=require('fs');
const path=require('path');
const src=path.resolve(__dirname,'../dist');
const dest=path.resolve(__dirname,'../../pub/web');
if(!fs.existsSync(src)){
console.error('dist directory not found');
process.exit(1);
}
fs.rmSync(dest,{recursive:true,force:true});
fs.mkdirSync(dest,{recursive:true});
const copy=(from,to)=>{
for(const entry of fs.readdirSync(from,{withFileTypes:true})){
const srcPath=path.join(from,entry.name);
const destPath=path.join(to,entry.name);
if(entry.isDirectory()){
fs.mkdirSync(destPath,{recursive:true});
copy(srcPath,destPath);
}else{
fs.copyFileSync(srcPath,destPath);
}
}
};
copy(src,dest);

