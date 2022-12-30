<img src=".github/example.png" alt="Banner Service" />

## What is banner-service?

Banner Service is an easy-to-use CLI tool to generate banners for your GitHub repos. It supports fetching images from Unsplash to build your banners!

## How to use
1) Clone this repo
2) Go to [Unsplash Developers](https://unsplash.com/developers) and create an account
3) Create an application
4) Copy your access and secret keys from their dashboard
5) Rename `config.example.toml` to `config.toml`
6) That's it! All you need to do now is run the tool!

## Documentation
```
Usage: banner-service <QUERY> <TEXT> <FONT_SIZE> <BORDER_RADIUS> <DESCRIPTION> <DESCRIPTION_COLOR_OFFSET>

Arguments:
  <QUERY>
  <TEXT>
  <FONT_SIZE>
  <BORDER_RADIUS>
  <DESCRIPTION>
  <DESCRIPTION_COLOR_OFFSET>

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```

| Argument      | Description |
| -----------   | ----------- |
| query                             | The query to use when searching Unsplash e.g. "japan"       |
| text                              | The title text to use on the banner        |
| font_size                         | How big the title text should be. The description text size will be half this size        |
| border_radius                     | The radius of the rounded corners on the banner        |
| description                       | The description text       |
| description_color_offset          | What color the description text should be, offsetted from white `(255, 255, 255)`. For example, if `105` is given, the description color text will be `(150, 150, 150)`        |
