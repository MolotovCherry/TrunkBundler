# Trunk bundler

This is a post-build hook to [Trunk](https://github.com/thedodd/trunk/) whose purpose is to bundle and compile an es module project into 1 distributable js file

Features:
- Package your own project es modules (it also has node package support!) into one `dist.min.js` file

Note: Only tested on Windows so far, but should be able to support Linux. However, currently I think it will probably fail on Linux. Please submit a bug report as they come, it should be fairly quick to solve. 🙂

## To use:
### Add to `Trunk.toml`
```toml
[[hooks]]
stage = "post_build"
command = "/path/to/trunk_bundler.exe"
```

### Add these in `index.html`
Using a js esm project, and optionally node module packages and external libs - it all gets bundled together into 1 js file:
- You need to specify `data-bundler` to activate it.
- `href` is relative to the location of the `index.html` file it's defined in
- `data-output` is the resulting file you want the output to appear in. The path is relative to your `dist` folder
- `data-modules` specifies the root module of your js project. This can be a comma separated list as well such as `app, foo`
- `data-preload` specifies you want to use a `<link rel="preload" href="{url}" as="script" />` tag
- `data-async` specifies you want to use `async` attribute on the script tag
- `data-defer` specifies you want to use `defer` attribute on the script tag
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
