# motivation

npm why みたいなかんじで、 why <command> と打つと、パスにあるそのコマンドはどんな方法(どのパッケージマネージャー)でインストールされたものか(brewとかnpm -g とか)を特定してくれるコマンドがほしい

# how 

rust で実装してみたい。

# requirements

CLIツール。
Windows, mac, linuxに対応したい
各OSごとにパッケージマネージャーの特定の手掛かりのデータベース(formatはJSONとか?)を持ち、追加していけるようにしたい。
できれば、そのパッケージを特定する情報を各パッケージマネージャーのコマンドを起動して表示できるといい。

各OS共通なら、npm -g, bun -g 
macOSなら、 brew 、 インストーラーファイル(dmgとか)
windowsなら、chocolatey
Debian/Ubutuntuなら、apt

もちろん、セルフビルドした場合など、不明なものも多いでしょう。OS標準であることが明らかならそういう表示も可能かな。

