TERMUX_PKG_DESCRIPTION="Starkli (/ˈstɑːrklaɪ/), a blazing fast CLI tool for Starknet powered by starknet-rs"
TERMUX_PKG_LICENSE="MPL-2.0"
TERMUX_PKG_MAINTAINER="Jonathan Lei <me@xjonathan.dev>"
TERMUX_PKG_VERSION="0.0.0"
TERMUX_PKG_SRCURL="git+/home/builder/termux-packages/starkli"
TERMUX_PKG_BUILD_IN_SRC=true

termux_step_make() {
	termux_setup_rust
	cargo build --jobs ${TERMUX_PKG_MAKE_PROCESSES} --target ${CARGO_TARGET_NAME} --release
}

termux_step_make_install() {
	install -Dm700 -t ${TERMUX_PREFIX}/bin target/${CARGO_TARGET_NAME}/release/starkli

	install -Dm644 /dev/null ${TERMUX_PREFIX}/share/bash-completion/completions/starkli.bash
	install -Dm644 /dev/null ${TERMUX_PREFIX}/share/zsh/site-functions/_starkli
	install -Dm644 /dev/null ${TERMUX_PREFIX}/share/fish/vendor_completions.d/starkli.fish
}

termux_step_create_debscripts() {
	cat <<-EOF >./postinst
		#!${TERMUX_PREFIX}/bin/sh
		starkli setup --generate-completion bash > ${TERMUX_PREFIX}/share/bash-completion/completions/starkli.bash
		starkli setup --generate-completion zsh > ${TERMUX_PREFIX}/share/zsh/site-functions/_starkli
		starkli setup --generate-completion fish > ${TERMUX_PREFIX}/share/fish/vendor_completions.d/starkli.fish
	EOF
}
