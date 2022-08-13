if [[ ($OSTYPE == 'darwin'*)  ]]; then
  if [[ $(brew list | grep util-linux) != "util-linux" ]]; then
    # некоторые утилиты macos отличаются от linux - ставим аналоги
    brew install util-linux
  fi
    export PATH=$(brew --prefix util-linux)/bin/:$PATH
fi