# Resources

This directory contains files that will be added to the executable during compilation and other files required to create a install package.


pkexec update-alternatives --set x-www-browser /home/sheep/development/browsewith_rust/volumes/app/bin/linux/browsewith

update-alternatives --set x-www-browser /usr/bin/firefox

update-alternatives --query x-www-browser

update-alternatives --install /usr/bin/x-www-browser x-www-browser /home/sheep/development/browsewith_rust/volumes/app/bin/linux/browsewith 50


# display in appfinder
./local/share/applications/browsewith.desktop

--install/configure

if SUPER_USER
  cp browsewith /usr/bin/browsewith
  save 'browsewith.desktop' to '/usr/share/applications/`
  save 'browsewith.ico' to '/usr/share/icons/'
else
  edit 'browsewith.desktop' in memory
    - change Exec=
    - change Icon=
  save 'browsewith.desktop' to '~/.local/applications/'
  save 'browsewith.ico' to '~/.browsewith/'
end_if

if no '~/.browsewith/browsewith.conf'
  mkdir '~/.browsewith'
  scan for browsers
  save 'browsewith.conf' to '~/.browsewith/'
end_if
