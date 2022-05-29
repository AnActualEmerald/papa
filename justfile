set shell := ["fish", "-c"]
set export 

@draft t:
    @cargo build --release
    @cargo deb
    @gh release create {{t}} --notes "# Papa {{t}}" -d --title "Papa {{t}}"
    @fish -c "gh release upload {{t}} target/debian/papa_(string replace 'v' '' {{t}})_amd64.deb"
    @gh release upload {{t}} target/release/papa
    
    
