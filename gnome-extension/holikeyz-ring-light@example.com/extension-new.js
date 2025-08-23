import GObject from 'gi://GObject';
import St from 'gi://St';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import Clutter from 'gi://Clutter';
import Pango from 'gi://Pango';

import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import * as Slider from 'resource:///org/gnome/shell/ui/slider.js';

const DBUS_NAME = 'com.elgato.RingLight';
const DBUS_PATH = '/com/elgato/RingLight';
const DBUS_INTERFACE = 'com.elgato.RingLight.Control';

const ElgatoDBusInterface = `
<node>
  <interface name="${DBUS_INTERFACE}">
    <method name="TurnOn">
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="TurnOff">
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="Toggle">
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="SetBrightness">
      <arg type="y" direction="in" name="brightness"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="SetTemperature">
      <arg type="u" direction="in" name="kelvin"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="GetStatus">
      <arg type="b" direction="out" name="is_on"/>
      <arg type="y" direction="out" name="brightness"/>
      <arg type="u" direction="out" name="temperature"/>
    </method>
    <method name="Identify">
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="ApplyScene">
      <arg type="s" direction="in" name="scene"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <property name="IsOn" type="b" access="read"/>
    <property name="Brightness" type="y" access="readwrite"/>
    <property name="Temperature" type="u" access="readwrite"/>
  </interface>
</node>`;

const ElgatoDBusProxy = Gio.DBusProxy.makeProxyWrapper(ElgatoDBusInterface);

// Scene definitions
const SCENES = [
    {
        id: 'daylight',
        name: 'Daylight',
        description: 'Bright & Focused',
        brightness: 80,
        temperature: 5600,
        icon: 'weather-clear-symbolic',
        color: '#FFE4B5'
    },
    {
        id: 'warm',
        name: 'Warm',
        description: 'Cozy Evening',
        brightness: 60,
        temperature: 3200,
        icon: 'weather-sunset-symbolic',
        color: '#FFA500'
    },
    {
        id: 'cool',
        name: 'Cool',
        description: 'Modern & Crisp',
        brightness: 70,
        temperature: 6500,
        icon: 'weather-snow-symbolic',
        color: '#E0FFFF'
    },
    {
        id: 'reading',
        name: 'Reading',
        description: 'Perfect for Focus',
        brightness: 90,
        temperature: 4500,
        icon: 'accessories-text-editor-symbolic',
        color: '#FFFACD'
    },
    {
        id: 'video',
        name: 'Video',
        description: 'Content Creation',
        brightness: 75,
        temperature: 5000,
        icon: 'camera-video-symbolic',
        color: '#F0F8FF'
    },
    {
        id: 'relax',
        name: 'Relax',
        description: 'Wind Down',
        brightness: 40,
        temperature: 2900,
        icon: 'night-light-symbolic',
        color: '#FFB6C1'
    }
];

const ElgatoIndicator = GObject.registerClass(
class ElgatoIndicator extends PanelMenu.Button {
    _init() {
        super._init(0.0, 'Elgato Ring Light');
        
        this._isConnected = false;
        this._isOn = false;
        this._brightness = 50;
        this._temperature = 4500;
        this._currentScene = null;
        this._sceneButtons = new Map();
        
        // Panel icon
        let icon = new St.Icon({
            icon_name: 'weather-clear-symbolic',
            style_class: 'system-status-icon elgato-panel-button',
        });
        this.add_child(icon);
        this._icon = icon;
        
        // Load stylesheet
        this._loadStylesheet();
        
        // Build menu
        this._buildMenu();
        
        // Connect to D-Bus
        this._connectToDBus();
        
        // Cleanup on destroy
        this.connect('destroy', () => {
            this._onDestroy();
        });
    }
    
    _loadStylesheet() {
        let extensionPath = GLib.get_home_dir() + '/.local/share/gnome-shell/extensions/elgato-ring-light@example.com';
        let stylesheetPath = extensionPath + '/stylesheet.css';
        let stylesheetFile = Gio.File.new_for_path(stylesheetPath);
        
        if (stylesheetFile.query_exists(null)) {
            let theme = St.ThemeContext.get_for_stage(global.stage).get_theme();
            theme.load_stylesheet(stylesheetFile);
        }
    }
    
    _buildMenu() {
        // Main container with custom styling
        this.menu.box.add_style_class_name('elgato-menu');
        
        // Header with power toggle
        let header = new PopupMenu.PopupBaseMenuItem({ 
            reactive: false,
            can_focus: false
        });
        header.add_style_class_name('elgato-header');
        
        let headerBox = new St.BoxLayout({ 
            vertical: false,
            style_class: 'elgato-power-row'
        });
        
        // Power icon
        let powerIcon = new St.Icon({
            icon_name: 'system-shutdown-symbolic',
            style_class: 'elgato-power-icon'
        });
        headerBox.add_child(powerIcon);
        
        // Power switch
        this._powerSwitch = new PopupMenu.Switch(false);
        this._powerSwitch.connect('notify::state', () => {
            this._toggleLight(this._powerSwitch.state);
        });
        headerBox.add_child(this._powerSwitch);
        
        // Light name
        let lightLabel = new St.Label({ 
            text: 'Ring Light',
            y_align: Clutter.ActorAlign.CENTER,
            style: 'font-weight: bold; font-size: 14px;'
        });
        headerBox.add_child(lightLabel);
        
        // Status label
        this._statusLabel = new St.Label({
            text: 'Connecting...',
            style_class: 'elgato-status-label',
            y_align: Clutter.ActorAlign.CENTER
        });
        headerBox.add_child(this._statusLabel);
        
        header.add_child(headerBox);
        this.menu.addMenuItem(header);
        
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        
        // Brightness control
        let brightnessSection = new PopupMenu.PopupBaseMenuItem({ 
            reactive: false,
            can_focus: false
        });
        brightnessSection.add_style_class_name('elgato-control-section');
        
        let brightnessBox = new St.BoxLayout({ 
            vertical: false,
            style_class: 'elgato-slider-row'
        });
        
        let brightnessIcon = new St.Icon({
            icon_name: 'display-brightness-symbolic',
            icon_size: 16
        });
        brightnessBox.add_child(brightnessIcon);
        
        let brightnessLabel = new St.Label({ 
            text: 'Brightness',
            style_class: 'elgato-slider-label',
            y_align: Clutter.ActorAlign.CENTER 
        });
        brightnessBox.add_child(brightnessLabel);
        
        this._brightnessSlider = new Slider.Slider(0.5);
        this._brightnessSlider.add_style_class_name('elgato-slider');
        this._brightnessTimeout = null;
        
        this._brightnessSlider.connect('notify::value', (slider) => {
            let brightness = Math.round(slider.value * 100);
            this._brightnessValueLabel.text = `${brightness}%`;
            
            if (this._brightnessTimeout) {
                GLib.source_remove(this._brightnessTimeout);
            }
            
            this._brightnessTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 500, () => {
                this._setBrightness(brightness);
                this._brightnessTimeout = null;
                return GLib.SOURCE_REMOVE;
            });
        });
        
        brightnessBox.add_child(this._brightnessSlider);
        
        this._brightnessValueLabel = new St.Label({ 
            text: '50%',
            style_class: 'elgato-slider-value',
            y_align: Clutter.ActorAlign.CENTER 
        });
        brightnessBox.add_child(this._brightnessValueLabel);
        
        brightnessSection.add_child(brightnessBox);
        this.menu.addMenuItem(brightnessSection);
        
        // Temperature control
        let temperatureSection = new PopupMenu.PopupBaseMenuItem({ 
            reactive: false,
            can_focus: false
        });
        temperatureSection.add_style_class_name('elgato-control-section');
        
        let temperatureBox = new St.BoxLayout({ 
            vertical: false,
            style_class: 'elgato-slider-row'
        });
        
        let temperatureIcon = new St.Icon({
            icon_name: 'preferences-color-symbolic',
            icon_size: 16
        });
        temperatureBox.add_child(temperatureIcon);
        
        let temperatureLabel = new St.Label({ 
            text: 'Temperature',
            style_class: 'elgato-slider-label',
            y_align: Clutter.ActorAlign.CENTER 
        });
        temperatureBox.add_child(temperatureLabel);
        
        this._temperatureSlider = new Slider.Slider(0.5);
        this._temperatureSlider.add_style_class_name('elgato-slider');
        this._temperatureTimeout = null;
        
        this._temperatureSlider.connect('notify::value', (slider) => {
            let kelvin = Math.round(2900 + (slider.value * (7000 - 2900)));
            this._temperatureValueLabel.text = `${kelvin}K`;
            
            if (this._temperatureTimeout) {
                GLib.source_remove(this._temperatureTimeout);
            }
            
            this._temperatureTimeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 500, () => {
                this._setTemperature(kelvin);
                this._temperatureTimeout = null;
                return GLib.SOURCE_REMOVE;
            });
        });
        
        temperatureBox.add_child(this._temperatureSlider);
        
        this._temperatureValueLabel = new St.Label({ 
            text: '4500K',
            style_class: 'elgato-slider-value',
            y_align: Clutter.ActorAlign.CENTER 
        });
        temperatureBox.add_child(this._temperatureValueLabel);
        
        temperatureSection.add_child(temperatureBox);
        this.menu.addMenuItem(temperatureSection);
        
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        
        // Scenes section
        this._buildScenesGrid();
        
        // Quick actions
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        
        let actionsBox = new St.BoxLayout({
            vertical: false,
            style_class: 'elgato-quick-actions'
        });
        
        let identifyButton = new St.Button({
            label: 'Identify',
            style_class: 'elgato-action-button',
            x_expand: true
        });
        identifyButton.connect('clicked', () => {
            this._identifyLight();
        });
        actionsBox.add_child(identifyButton);
        
        let settingsButton = new St.Button({
            label: 'Settings',
            style_class: 'elgato-action-button',
            x_expand: true
        });
        settingsButton.connect('clicked', () => {
            // TODO: Open settings dialog
            log('Settings clicked');
        });
        actionsBox.add_child(settingsButton);
        
        let actionsItem = new PopupMenu.PopupBaseMenuItem({ 
            reactive: false,
            can_focus: false
        });
        actionsItem.add_child(actionsBox);
        this.menu.addMenuItem(actionsItem);
    }
    
    _buildScenesGrid() {
        // Title
        let titleItem = new PopupMenu.PopupBaseMenuItem({ 
            reactive: false,
            can_focus: false
        });
        let titleLabel = new St.Label({
            text: 'Scenes',
            style_class: 'elgato-scenes-title'
        });
        titleItem.add_child(titleLabel);
        this.menu.addMenuItem(titleItem);
        
        // Scene grid container
        let scenesContainer = new PopupMenu.PopupBaseMenuItem({ 
            reactive: false,
            can_focus: false
        });
        scenesContainer.add_style_class_name('elgato-scenes-section');
        
        let scenesGrid = new St.BoxLayout({
            vertical: true,
            style_class: 'elgato-scenes-grid'
        });
        
        // Create scene buttons in a 2-column grid
        for (let i = 0; i < SCENES.length; i += 2) {
            let row = new St.BoxLayout({
                vertical: false,
                style_class: 'elgato-scene-row'
            });
            
            // Add two scenes per row
            for (let j = i; j < Math.min(i + 2, SCENES.length); j++) {
                let scene = SCENES[j];
                let button = this._createSceneButton(scene);
                this._sceneButtons.set(scene.id, button);
                row.add_child(button);
            }
            
            scenesGrid.add_child(row);
        }
        
        scenesContainer.add_child(scenesGrid);
        this.menu.addMenuItem(scenesContainer);
    }
    
    _createSceneButton(scene) {
        let button = new St.Button({
            style_class: 'elgato-scene-button',
            can_focus: true
        });
        
        // Scene content with background
        let content = new St.BoxLayout({
            vertical: true,
            style_class: 'elgato-scene-content',
            style: this._getSceneBackgroundStyle(scene)
        });
        
        // Overlay for text
        let overlay = new St.BoxLayout({
            vertical: true,
            style_class: 'elgato-scene-overlay'
        });
        
        // Spacer to push text to bottom
        overlay.add_child(new St.Widget({ y_expand: true }));
        
        // Scene info
        let info = new St.BoxLayout({
            vertical: true,
            style_class: 'elgato-scene-info'
        });
        
        let nameLabel = new St.Label({
            text: scene.name,
            style_class: 'elgato-scene-name'
        });
        info.add_child(nameLabel);
        
        let descLabel = new St.Label({
            text: scene.description,
            style_class: 'elgato-scene-description'
        });
        info.add_child(descLabel);
        
        overlay.add_child(info);
        content.add_child(overlay);
        button.set_child(content);
        
        // Connect click handler
        button.connect('clicked', () => {
            this._applyScene(scene.id);
            this._updateSceneButtons(scene.id);
        });
        
        return button;
    }
    
    _getSceneBackgroundStyle(scene) {
        // Check if image exists
        let extensionPath = GLib.get_home_dir() + '/.local/share/gnome-shell/extensions/elgato-ring-light@example.com';
        let imagePath = `${extensionPath}/images/${scene.id}.jpg`;
        let imageFile = Gio.File.new_for_path(imagePath);
        
        if (imageFile.query_exists(null)) {
            return `background-image: url("file://${imagePath}");`;
        } else {
            // Fallback to gradient
            return this._getSceneGradient(scene);
        }
    }
    
    _getSceneGradient(scene) {
        switch(scene.id) {
            case 'daylight':
                return 'background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);';
            case 'warm':
                return 'background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);';
            case 'cool':
                return 'background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);';
            case 'reading':
                return 'background: linear-gradient(135deg, #43e97b 0%, #38f9d7 100%);';
            case 'video':
                return 'background: linear-gradient(135deg, #fa709a 0%, #fee140 100%);';
            case 'relax':
                return 'background: linear-gradient(135deg, #30cfd0 0%, #330867 100%);';
            default:
                return 'background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);';
        }
    }
    
    _updateSceneButtons(activeSceneId) {
        this._currentScene = activeSceneId;
        this._sceneButtons.forEach((button, sceneId) => {
            if (sceneId === activeSceneId) {
                button.add_style_class_name('active');
            } else {
                button.remove_style_class_name('active');
            }
        });
    }
    
    _connectToDBus() {
        try {
            this._proxy = new ElgatoDBusProxy(
                Gio.DBus.session,
                DBUS_NAME,
                DBUS_PATH,
                (proxy, error) => {
                    if (error) {
                        log(`Failed to connect to Elgato D-Bus service: ${error}`);
                        this._setConnectionStatus(false);
                    } else {
                        this._setConnectionStatus(true);
                        this._updateStatus();
                    }
                }
            );
        } catch (e) {
            log(`Error creating D-Bus proxy: ${e}`);
            this._setConnectionStatus(false);
        }
    }
    
    _setConnectionStatus(connected) {
        this._isConnected = connected;
        if (connected) {
            this._statusLabel.text = 'Connected';
            this._statusLabel.remove_style_class_name('elgato-connecting');
            this._icon.icon_name = 'weather-clear-symbolic';
            this._powerSwitch.reactive = true;
            this._brightnessSlider.reactive = true;
            this._temperatureSlider.reactive = true;
        } else {
            this._statusLabel.text = 'Disconnected';
            this._statusLabel.add_style_class_name('elgato-connecting');
            this._icon.icon_name = 'dialog-error-symbolic';
            this._powerSwitch.reactive = false;
            this._brightnessSlider.reactive = false;
            this._temperatureSlider.reactive = false;
        }
    }
    
    _updateStatus() {
        if (!this._isConnected || !this._proxy) return;
        
        this._proxy.GetStatusRemote((result, error) => {
            if (error) {
                log(`Failed to get status: ${error}`);
                return;
            }
            
            let [is_on, brightness, temperature] = result;
            this._isOn = is_on;
            this._brightness = brightness;
            this._temperature = temperature;
            
            this._powerSwitch.state = is_on;
            this._brightnessSlider.value = brightness / 100;
            this._brightnessValueLabel.text = `${brightness}%`;
            
            let tempNorm = (temperature - 2900) / (7000 - 2900);
            this._temperatureSlider.value = tempNorm;
            this._temperatureValueLabel.text = `${temperature}K`;
            
            if (is_on) {
                this._icon.icon_name = 'weather-clear-symbolic';
                this._statusLabel.text = 'On';
            } else {
                this._icon.icon_name = 'weather-clear-night-symbolic';
                this._statusLabel.text = 'Off';
            }
            
            // Check if current state matches a scene
            this._detectCurrentScene(brightness, temperature);
        });
    }
    
    _detectCurrentScene(brightness, temperature) {
        for (let scene of SCENES) {
            if (Math.abs(scene.brightness - brightness) < 5 && 
                Math.abs(scene.temperature - temperature) < 200) {
                this._updateSceneButtons(scene.id);
                return;
            }
        }
        // No matching scene
        this._updateSceneButtons(null);
    }
    
    _toggleLight(state) {
        if (!this._isConnected || !this._proxy) return;
        
        let method = state ? 'TurnOnRemote' : 'TurnOffRemote';
        this._proxy[method]((result, error) => {
            if (error) {
                log(`Failed to toggle light: ${error}`);
            } else {
                this._updateStatus();
            }
        });
    }
    
    _setBrightness(brightness) {
        if (!this._isConnected || !this._proxy) {
            return;
        }
        
        this._proxy.SetBrightnessRemote(brightness, (result, error) => {
            if (error) {
                log(`Failed to set brightness: ${error}`);
            }
        });
    }
    
    _setTemperature(kelvin) {
        if (!this._isConnected || !this._proxy) return;
        
        this._proxy.SetTemperatureRemote(kelvin, (result, error) => {
            if (error) {
                log(`Failed to set temperature: ${error}`);
            }
        });
    }
    
    _applyScene(sceneId) {
        if (!this._isConnected || !this._proxy) return;
        
        this._proxy.ApplySceneRemote(sceneId, (result, error) => {
            if (error) {
                log(`Failed to apply scene: ${error}`);
            } else {
                this._updateStatus();
            }
        });
    }
    
    _identifyLight() {
        if (!this._isConnected || !this._proxy) return;
        
        this._proxy.IdentifyRemote((result, error) => {
            if (error) {
                log(`Failed to identify light: ${error}`);
            }
        });
    }
    
    _onDestroy() {
        if (this._brightnessTimeout) {
            GLib.source_remove(this._brightnessTimeout);
            this._brightnessTimeout = null;
        }
        if (this._temperatureTimeout) {
            GLib.source_remove(this._temperatureTimeout);
            this._temperatureTimeout = null;
        }
        this._proxy = null;
    }
});

export default class ElgatoExtension {
    constructor() {
        this._indicator = null;
    }
    
    enable() {
        this._indicator = new ElgatoIndicator();
        Main.panel.addToStatusArea('elgato-indicator', this._indicator);
    }
    
    disable() {
        if (this._indicator) {
            this._indicator.destroy();
            this._indicator = null;
        }
    }
}