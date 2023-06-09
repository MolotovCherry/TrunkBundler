# Trunk bundler

This is a post-build hook to [Trunk](https://github.com/thedodd/trunk/) whose purpose is to minify the generated html and js output.

Features:
- Automatically minifies app js/html in release mode
- You can also package your own project es modules (it also has node package support!) into one `dist.min.js` file

## To use:
- Add to `Trunk.toml`
```toml
[[hooks]]
stage = "post_build"
command = "/path/to/post_bundler.exe"
```
- Add to your `index.html` (or equivalent) if you want to bundle and include a js project. This project can be a regular es module project, and can also optionally use node module packages and external libs you provide.
- - You need to specify `data-bundler` to activate it.
- - `href` is relative to the location of the `index.html` file it's defined in
- - `data-output` is the resulting file you want the output to appear in. The path is relative to your `dist` folder
- - `data-modules` specifies the root module of your js project. This can be a comma separated list as well such as `app, foo`
```html
<link data-bundler rel="js" href="../static/scripts" data-modules="app" data-output="static/dist.min.js" />
```
- Your js project setup should look like this
- - `package.json`/`node_modules` is optional; it is the file for managing your `node_modules`
- - `lib` is optional. Places named folders inside this; all js files inside the folders will be included in the output js.
- - `src` is where your js files/modules live. If you specify `data-modules="app"`, it will look in `<href path>/src/<module name>.{js,ts,jsx,tsx}` for the module.
- - It will also look in `<href path>/<module name>.{js,ts,jsx,tsx}` for the module if you only need a simpler setup without all the folders.
```
│   package.json
│   app.js
│
├───lib
│   │
│   ├───lib1
│   │       file1.js
│   │       file2.js
│   │
│   └───lib2
│           file1.js
│           file2.js
│
├───node_modules
│   │ ...
│
└───src
        app.js
```

This builder is also capable of building javascript, typescript, jsx, and tsx code.
