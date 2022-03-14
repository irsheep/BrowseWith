# BrowseWith

BrowseWith is an application which allows the user to select a web browser before opening a URL from an application, such as clicking on a URL in an email client. Alternatively it can be also placed on the taskbar so the user can choose which browser to use.

> **ATTENTION** BrowseWith is currently in **alpha**, it is fully functional and working, but it does not handle errors, such as files missing, invalid configuration, etc.

![BrowseWith Windows](/images/browsewith_windows.png)

## Runtime Arguments

You can run BrowseWith with some arguments to setup BrowseWith on the system.

**--install**: Installs BrowseWith by copying the files to the appropriate locations and register itself as a handler for the HTTP and HTTPS protocols. If *--install* is executed with elevated privileges then it is installed for all users.

**--uninstall**: Removes BrowseWith from the system all and its files, including the configuration file.

> **Note** the *install* and *uninstall* arguments can be run as a privileged user or as normal user. If running as a privileged user then BrowseWith will be installed for all users on the system; if running as normal user then it will affect only the current user.

**--set-as-default-browser**: Configures the system to use BrowseWith as the default browser. This is a per user setting. On Windows this will open the *Default apps* application, you can then set BrowseWith as the default web browser; alternatively you can click on **Choose default apps by protocol** and associate BrowseWith with certain protocols only (HTTP and HTTPS for example).

**--status**: Displays the current default web browser and where the application files are/will be installed and the location of the configuration file.

# Configuration

BrowseWith will create the configuration file when it runs, if the configuration file doesn't exist at the required location, *~/.config/browsewith/config.json* for Linux/BSD, *%userprofile%\.browsewith\config.json* on Windows.

Default configuration
```json
{
  "settings": {
      "homepage": "about:blank",
      "host_info": true,
      "buttons": {
          "width": 180,
          "height": 70,
          "spacing": 5,
          "per_row": 3,
          "show_label": true,
          "show_image": true,
          "image_position": "left"
      },
      "window": {
          "always_ontop": true,
          "position": "center"
      }
  },
  "browsers_list": []
}
```

### Main settings
- **homepage**: URL to open if no URL is passed as argument.
- **host_info**: [true, false] Displays the URL that will be opened.

### Buttons settings
- **width**: Button width in pixels.
- **height**: Button height in pixels.
- **spacing**: Number of pixel to separate each button.
- **per_row**: Number of buttons per row.
- **show_label**: [true, false] Show or hide the *title* of each button.
- **show_image**: [true, false] Show or hide the icons for the buttons.
- **image_position**: [left, top, bottom, right] where to display the icon in relation to the label.

### Application Window settings
- **always_ontop**: [true, false] Make BrowseWith to be always visible on top of other windows.
- **position**: [none, center, mouse] Initial placement of the window, *none* decided by the OS, *center* centre of the screen, *mouse* cantered on the mouse pointer.

### Browsers
BrowseWith will try and detect the browsers installed on the system; this is only done if the configuration file isn't present. So if another browser is installed then it needs to be manually added to the **browser_list** section in the configuration file. 

BrowseWith displays the browsers in the application in the same order they are in the **browser_list**. 

```json
"browser_list": [
  {
    "title": "_Brave",
    "executable": "C:\\Program Files\\BraveSoftware\\Brave-Browser\\Application\\brave.exe",
    "arguments": "",
    "icon": "C:\\Program Files\\BraveSoftware\\Brave-Browser\\Application\\brave.exe,0"
  }
]
```

- **title**: Label to be associated with the button. You can use an underscore (_) to associate an hotkey with the button. For example if the title is set to **"Hello W_orld"** pressing **ALT+o** would activate the button.
- **executable**: Full path to the application executable file.
- **arguments**: One or more arguments to the passed to the application.
- **icon**: Full path to the location of the icon to associate with the button.
