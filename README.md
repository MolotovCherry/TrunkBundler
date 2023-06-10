# Trunk bundler

This is a post-build hook to [Trunk](https://github.com/thedodd/trunk/) whose purpose is to minify the generated html and js output.

Features:
- Automatically minifies app js/html in release mode (sass/scss/css support will come eventually)
- You can also package your own project es modules (it also has node package support!) into one `dist.min.js` file

Note: Only tested on Windows so far, but should be able to support Linux. However, currently I think it will probably fail on Linux. Please submit a bug report as they come, it should be fairly quick to solve. 🙂

Note 2: It seems that [due to a bug in swc](https://github.com/swc-project/swc/issues/7513), minify had to be disabled on app.js output as it was incorrect. Hopefully we can re-enable it soon

## To use:
### Add to `Trunk.toml`
```toml
[[hooks]]
stage = "post_build"
command = "/path/to/trunk_bundler.exe"
```

### Add these in `index.html`
In order to minify the wasm js file
- Set `data-package` to the package name of your wasm app. This is needed to know which js name to glob for

`<link data-bundler rel="app" data-package="<name here>" />`

If you want to bundle and include a js project, keep reading. You can have a regular js esm project, and can also optionally use node module packages and external libs, and it all gets bundled together into 1 js file. If you want to do that, follow the below instructions:
- You need to specify `data-bundler` to activate it.
- `href` is relative to the location of the `index.html` file it's defined in
- `data-output` is the resulting file you want the output to appear in. The path is relative to your `dist` folder
- `data-modules` specifies the root module of your js project. This can be a comma separated list as well such as `app, foo`
- You are allowed to have multiple ones of these, which would result in multiple js outputs.
```html
<link data-bundler rel="js" href="../static/scripts" data-modules="app" data-output="static/dist.min.js" />
```

### Final project setup
- `package.json`/`node_modules` is optional
- `lib` is optional. Place named folders inside this; all js files inside the folders will be included in the output js. The file names can be anything, and the folder structure inside does not matter.
- `src` is where your js files/modules live. If you specify `data-modules="app"`, it will look in `<href path>/src/<module name>.{js,ts,jsx,tsx}` for the module.
- It will also look in `<href path>/<module name>.{js,ts,jsx,tsx}` for the module if you would rather have a simpler setup.
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

## Final notes
This builder is also capable of building javascript, typescript, jsx, and tsx code.

Also, all js modules/projects are built in parallel across multiple threads if you have multiple `<link>` tags
