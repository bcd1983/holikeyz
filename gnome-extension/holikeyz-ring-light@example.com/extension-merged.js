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

const DBUS_NAME = 'com.holikeyz.RingLight';
const DBUS_PATH = '/com/holikeyz/RingLight';
const DBUS_INTERFACE = 'com.holikeyz.RingLight.Control';

const HolikeyzDBusInterface = `
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
    <method name="TurnOnLight">
      <arg type="y" direction="in" name="light_index"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="TurnOffLight">
      <arg type="y" direction="in" name="light_index"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="ToggleLight">
      <arg type="y" direction="in" name="light_index"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="SetBrightness">
      <arg type="y" direction="in" name="brightness"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="SetBrightnessLight">
      <arg type="y" direction="in" name="brightness"/>
      <arg type="y" direction="in" name="light_index"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="SetTemperature">
      <arg type="u" direction="in" name="kelvin"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="SetTemperatureLight">
      <arg type="u" direction="in" name="kelvin"/>
      <arg type="y" direction="in" name="light_index"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="GetStatus">
      <arg type="b" direction="out" name="is_on"/>
      <arg type="y" direction="out" name="brightness"/>
      <arg type="u" direction="out" name="temperature"/>
    </method>
    <method name="GetNumLights">
      <arg type="y" direction="out" name="num_lights"/>
    </method>
    <method name="GetAllLightsStatus">
      <arg type="ab" direction="out" name="is_on_array"/>
      <arg type="ay" direction="out" name="brightness_array"/>
      <arg type="au" direction="out" name="temperature_array"/>
    </method>
    <method name="GetLightStatus">
      <arg type="y" direction="in" name="light_index"/>
      <arg type="b" direction="out" name="is_on"/>
      <arg type="y" direction="out" name="brightness"/>
      <arg type="u" direction="out" name="temperature"/>
    </method>
    <method name="GetAccessoryInfo">
      <arg type="s" direction="out" name="product_name"/>
      <arg type="s" direction="out" name="firmware_version"/>
      <arg type="s" direction="out" name="serial_number"/>
      <arg type="u" direction="out" name="firmware_build"/>
      <arg type="s" direction="out" name="display_name"/>
      <arg type="as" direction="out" name="features"/>
    </method>
    <method name="GetSettings">
      <arg type="y" direction="out" name="power_on_behavior"/>
      <arg type="y" direction="out" name="power_on_brightness"/>
      <arg type="u" direction="out" name="power_on_temperature"/>
      <arg type="u" direction="out" name="switch_on_ms"/>
      <arg type="u" direction="out" name="switch_off_ms"/>
      <arg type="u" direction="out" name="color_change_ms"/>
    </method>
    <method name="SetSettings">
      <arg type="y" direction="in" name="power_on_behavior"/>
      <arg type="y" direction="in" name="power_on_brightness"/>
      <arg type="u" direction="in" name="power_on_temperature"/>
      <arg type="u" direction="in" name="switch_on_ms"/>
      <arg type="u" direction="in" name="switch_off_ms"/>
      <arg type="u" direction="in" name="color_change_ms"/>
      <arg type="b" direction="out" name="success"/>
    </method>
    <method name="SetPowerOnSettings">
      <arg type="y" direction="in" name="behavior"/>
      <arg type="y" direction="in" name="brightness"/>
      <arg type="u" direction="in" name="temperature"/>
      <arg type="b" direction="out" name="success"/>
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
    <property name="NumLights" type="y" access="read"/>
  </interface>
</node>`;

const HolikeyzDBusProxy = Gio.DBusProxy.makeProxyWrapper(HolikeyzDBusInterface);

// Scene definitions with your nice UI
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
        icon: 'media-record-symbolic',
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

const RingLightIndicator = GObject.registerClass(
class RingLightIndicator extends PanelMenu.Button {
    _init() {
        super._init(0.0, 'Ring Light Controller');
        
        // Initialize properties
        this._dbusProxy = null;
        this._isOn = false;
        this._brightness = 50;
        this._temperature = 4500;
        this._numLights = 1;
        this._currentScene = null;
        this._sliderUpdateInProgress = false;
        this._ignoreSliderUpdate = false;
        this._individualLightSwitches = [];
        
        // Create the panel icon with custom styling
        this._createPanelIcon();
        
        // Connect to D-Bus
        this._connectToDbus();
        
        // Build the menu
        this._buildMenu();
        
        // Check for multiple lights
        this._checkNumLights();
        
        // Update initial status
        this._updateStatus();
    }
    
    _createPanelIcon() {
        // Create custom styled panel box
        let box = new St.BoxLayout({ 
            style_class: 'panel-status-menu-box',
            style: 'padding: 0 4px;'
        });
        
        // Create icon with custom styling
        this._icon = new St.Icon({
            icon_name: 'dialog-information-symbolic',
            style_class: 'system-status-icon',
            style: 'padding: 0 2px;'
        });
        
        // Add power indicator dot
        this._powerIndicator = new St.Widget({
            style: 'width: 6px; height: 6px; border-radius: 3px; background-color: #666; margin-left: 2px;',
            y_align: Clutter.ActorAlign.CENTER
        });
        
        box.add_child(this._icon);
        box.add_child(this._powerIndicator);
        this.add_child(box);
    }
    
    _connectToDbus() {
        try {
            this._dbusProxy = new HolikeyzDBusProxy(
                Gio.DBus.session,
                DBUS_NAME,
                DBUS_PATH
            );
            
            // Listen for property changes
            this._dbusProxy.connect('g-properties-changed', () => {
                this._updateStatus();
            });
        } catch (e) {
            log(`Failed to connect to D-Bus: ${e}`);
        }
    }
    
    _buildMenu() {
        // Clear existing menu
        this.menu.removeAll();
        
        // Add custom CSS for better styling
        let style = `
            .ring-light-header { 
                font-weight: bold; 
                font-size: 1.1em; 
                padding: 8px 12px;
                background: linear-gradient(90deg, #2E3440 0%, #3B4252 100%);
                color: #ECEFF4;
                border-radius: 8px 8px 0 0;
            }
            .scene-card {
                padding: 12px;
                margin: 4px;
                border-radius: 8px;
                transition: all 200ms ease;
            }
            .scene-card:hover {
                background-color: rgba(255, 255, 255, 0.1);
                transform: scale(1.02);
            }
            .scene-card-active {
                background-color: rgba(129, 161, 193, 0.3);
                border: 1px solid #81A1C1;
            }
            .slider-container {
                padding: 8px 16px;
            }
            .slider-label {
                min-width: 90px;
                font-weight: 500;
            }
            .slider-value {
                min-width: 60px;
                text-align: right;
                font-family: monospace;
            }
            .power-button {
                padding: 12px;
                font-size: 1.1em;
                font-weight: bold;
            }
        `;
        
        // Header with gradient background
        let headerItem = new PopupMenu.PopupMenuItem('', { 
            reactive: false,
            can_focus: false
        });
        
        let headerBox = new St.BoxLayout({
            vertical: false,
            x_expand: true,
            style: 'padding: 12px; background: linear-gradient(90deg, #2E3440 0%, #3B4252 100%); border-radius: 8px;'
        });
        
        let headerIcon = new St.Icon({
            icon_name: 'dialog-information-symbolic',
            style: 'color: #88C0D0; margin-right: 8px;'
        });
        
        let headerLabel = new St.Label({
            text: 'Ring Light Controller',
            style: 'color: #ECEFF4; font-weight: bold; font-size: 1.1em;'
        });
        
        headerBox.add_child(headerIcon);
        headerBox.add_child(headerLabel);
        headerItem.add_child(headerBox);
        this.menu.addMenuItem(headerItem);
        
        // Separator with custom style
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        
        // Power Control with large toggle
        let powerItem = new PopupMenu.PopupBaseMenuItem({ reactive: false });
        let powerBox = new St.BoxLayout({
            vertical: false,
            x_expand: true,
            style: 'padding: 12px;'
        });
        
        let powerLabel = new St.Label({
            text: 'Power',
            y_align: Clutter.ActorAlign.CENTER,
            style: 'font-size: 1.1em; font-weight: bold; min-width: 100px;'
        });
        
        this._powerSwitch = new PopupMenu.Switch(false);
        this._powerSwitch.style = 'scale-x: 1.2; scale-y: 1.2;';
        
        let powerButton = new St.Button({
            child: this._powerSwitch,
            style_class: 'toggle-switch',
            x_expand: false,
            can_focus: true
        });
        
        powerButton.connect('clicked', () => {
            this._togglePower(!this._powerSwitch.state);
        });
        
        powerBox.add_child(powerLabel);
        powerBox.add_child(new St.Widget({ x_expand: true }));
        powerBox.add_child(powerButton);
        
        powerItem.add_child(powerBox);
        this.menu.addMenuItem(powerItem);
        
        // Separator
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        
        // Brightness Slider with better styling
        this._createSlider('Brightness', '%', 0, 100, 
            (value) => {
                let brightness = Math.round(value);
                this._brightnessValue.text = `${brightness}%`;
                if (!this._sliderUpdateInProgress) {
                    this._sliderUpdateInProgress = true;
                    GLib.timeout_add(GLib.PRIORITY_DEFAULT, 50, () => {
                        this._setBrightness(brightness);
                        this._sliderUpdateInProgress = false;
                        return GLib.SOURCE_REMOVE;
                    });
                }
            },
            (slider, valueLabel) => {
                this._brightnessSlider = slider;
                this._brightnessValue = valueLabel;
            }
        );
        
        // Temperature Slider with better styling
        this._createSlider('Temperature', 'K', 2900, 7000,
            (value) => {
                let temp = Math.round(value);
                this._tempValue.text = `${temp}K`;
                
                // Update slider color based on temperature
                let color = this._temperatureToColor(temp);
                this._tempSlider.style = `color: ${color};`;
                
                if (!this._sliderUpdateInProgress) {
                    this._sliderUpdateInProgress = true;
                    GLib.timeout_add(GLib.PRIORITY_DEFAULT, 50, () => {
                        this._setTemperature(temp);
                        this._sliderUpdateInProgress = false;
                        return GLib.SOURCE_REMOVE;
                    });
                }
            },
            (slider, valueLabel) => {
                this._tempSlider = slider;
                this._tempValue = valueLabel;
            }
        );
        
        // Separator
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        
        // Scene Presets Section with cards
        let sceneHeaderItem = new PopupMenu.PopupMenuItem('Scene Presets', {
            reactive: false,
            can_focus: false,
            style_class: 'popup-menu-item'
        });
        sceneHeaderItem.label.style = 'font-weight: bold; padding-left: 12px;';
        this.menu.addMenuItem(sceneHeaderItem);
        
        // Create scene grid
        this._createSceneGrid();
        
        // Separator
        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
        
        // Multiple lights section will be added here if detected
        this._multiLightSection = new PopupMenu.PopupMenuSection();
        this.menu.addMenuItem(this._multiLightSection);
        
        // Advanced Settings Section
        this._createAdvancedSection();
    }
    
    _checkNumLights() {
        if (!this._dbusProxy) return;
        
        // Try to get number of lights (new feature)
        if (this._dbusProxy.GetNumLightsRemote) {
            this._dbusProxy.GetNumLightsRemote((result) => {
                if (result && result[0] && result[0] > 1) {
                    this._numLights = result[0];
                    this._addMultiLightControls();
                }
            });
        }
    }
    
    _addMultiLightControls() {
        // Clear existing multi-light section
        this._multiLightSection.removeAll();
        
        // Add header
        let headerItem = new PopupMenu.PopupMenuItem(`${this._numLights} Lights Detected`, {
            reactive: false,
            can_focus: false
        });
        headerItem.label.style = 'font-weight: bold; padding-left: 12px;';
        this._multiLightSection.addMenuItem(headerItem);
        
        // Add individual light toggles
        for (let i = 0; i < this._numLights; i++) {
            let lightItem = new PopupMenu.PopupBaseMenuItem({ reactive: false });
            let lightBox = new St.BoxLayout({
                vertical: false,
                x_expand: true,
                style: 'padding: 8px 12px;'
            });
            
            let lightLabel = new St.Label({
                text: `Light ${i + 1}`,
                y_align: Clutter.ActorAlign.CENTER,
                style: 'min-width: 80px;'
            });
            
            let lightSwitch = new PopupMenu.Switch(false);
            let switchButton = new St.Button({
                child: lightSwitch,
                style_class: 'toggle-switch',
                x_expand: false,
                can_focus: true
            });
            
            switchButton.connect('clicked', () => {
                this._toggleLight(i, !lightSwitch.state);
            });
            
            lightBox.add_child(lightLabel);
            lightBox.add_child(new St.Widget({ x_expand: true }));
            lightBox.add_child(switchButton);
            
            lightItem.add_child(lightBox);
            this._multiLightSection.addMenuItem(lightItem);
            
            // Store reference
            this._individualLightSwitches[i] = lightSwitch;
        }
        
        // Add separator after multi-light section
        this._multiLightSection.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());
    }
    
    _createSlider(label, unit, min, max, onChange, onCreated) {
        let sliderItem = new PopupMenu.PopupBaseMenuItem({ reactive: false });
        let sliderBox = new St.BoxLayout({
            vertical: false,
            x_expand: true,
            style: 'padding: 8px 16px;'
        });
        
        let sliderLabel = new St.Label({
            text: label,
            y_align: Clutter.ActorAlign.CENTER,
            style: 'min-width: 90px; font-weight: 500;'
        });
        
        let slider = new Slider.Slider(0.5);
        slider.style = 'min-width: 200px;';
        
        let valueLabel = new St.Label({
            text: `${Math.round((min + max) / 2)}${unit}`,
            y_align: Clutter.ActorAlign.CENTER,
            style: 'min-width: 60px; text-align: right; font-family: monospace;'
        });
        
        slider.connect('notify::value', () => {
            if (!this._ignoreSliderUpdate) {
                let value = min + (slider.value * (max - min));
                onChange(value);
            }
        });
        
        sliderBox.add_child(sliderLabel);
        sliderBox.add_child(slider);
        sliderBox.add_child(valueLabel);
        
        sliderItem.add_child(sliderBox);
        this.menu.addMenuItem(sliderItem);
        
        onCreated(slider, valueLabel);
    }
    
    _createSceneGrid() {
        // Create a 2x3 grid for scenes
        let sceneContainer = new PopupMenu.PopupBaseMenuItem({ 
            reactive: false,
            can_focus: false 
        });
        
        let sceneGrid = new St.Widget({
            layout_manager: new Clutter.GridLayout({
                column_spacing: 8,
                row_spacing: 8,
                column_homogeneous: true
            }),
            style: 'padding: 8px;'
        });
        
        this._sceneButtons = {};
        
        SCENES.forEach((scene, index) => {
            let col = index % 2;
            let row = Math.floor(index / 2);
            
            // Create scene card
            let card = new St.Button({
                style_class: 'scene-card',
                style: `
                    background: linear-gradient(135deg, ${scene.color}22, ${scene.color}44);
                    border: 1px solid ${scene.color}66;
                    padding: 12px;
                    border-radius: 8px;
                    min-width: 140px;
                `,
                can_focus: true,
                track_hover: true
            });
            
            let cardContent = new St.BoxLayout({
                vertical: true,
                x_align: Clutter.ActorAlign.CENTER
            });
            
            // Scene icon
            let icon = new St.Icon({
                icon_name: scene.icon,
                style: `color: ${scene.color}; margin-bottom: 4px;`,
                icon_size: 24
            });
            
            // Scene name
            let nameLabel = new St.Label({
                text: scene.name,
                style: 'font-weight: bold; font-size: 0.9em;'
            });
            
            // Scene description
            let descLabel = new St.Label({
                text: scene.description,
                style: 'font-size: 0.75em; color: #888;'
            });
            
            cardContent.add_child(icon);
            cardContent.add_child(nameLabel);
            cardContent.add_child(descLabel);
            card.set_child(cardContent);
            
            card.connect('clicked', () => {
                this._applyScene(scene);
            });
            
            sceneGrid.layout_manager.attach(card, col, row, 1, 1);
            this._sceneButtons[scene.id] = card;
        });
        
        sceneContainer.add_child(sceneGrid);
        this.menu.addMenuItem(sceneContainer);
    }
    
    _createAdvancedSection() {
        // Advanced settings submenu
        let advancedSection = new PopupMenu.PopupSubMenuMenuItem('Advanced Settings', true);
        advancedSection.icon.icon_name = 'preferences-system-symbolic';
        
        // Device Info
        let infoItem = new PopupMenu.PopupMenuItem('Device Information');
        infoItem.connect('activate', () => {
            this._showDeviceInfo();
        });
        advancedSection.menu.addMenuItem(infoItem);
        
        // Power-on Settings
        let powerOnItem = new PopupMenu.PopupMenuItem('Power-On Behavior');
        powerOnItem.connect('activate', () => {
            this._showPowerOnSettings();
        });
        advancedSection.menu.addMenuItem(powerOnItem);
        
        // Identify Light
        let identifyItem = new PopupMenu.PopupMenuItem('Identify Light (Flash)');
        identifyItem.connect('activate', () => {
            this._identifyLight();
        });
        advancedSection.menu.addMenuItem(identifyItem);
        
        this.menu.addMenuItem(advancedSection);
    }
    
    _temperatureToColor(temp) {
        // Convert temperature to RGB color for visual feedback
        if (temp <= 3200) return '#FFA500'; // Warm orange
        if (temp >= 6500) return '#E0FFFF'; // Cool blue
        if (temp >= 5600) return '#FFE4B5'; // Daylight
        if (temp >= 4500) return '#FFFACD'; // Neutral
        return '#FFD700'; // Default golden
    }
    
    _setBrightness(brightness) {
        if (!this._dbusProxy) return;
        
        this._dbusProxy.SetBrightnessRemote(brightness, (result) => {
            if (result && result[0]) {
                this._updateSceneButtons(null);
            }
        });
    }
    
    _setTemperature(temperature) {
        if (!this._dbusProxy) return;
        
        this._dbusProxy.SetTemperatureRemote(temperature, (result) => {
            if (result && result[0]) {
                this._updateSceneButtons(null);
            }
        });
    }
    
    _togglePower(state) {
        if (!this._dbusProxy) return;
        
        if (state) {
            this._dbusProxy.TurnOnRemote((result) => {
                if (result && result[0]) {
                    this._updateStatus();
                }
            });
        } else {
            this._dbusProxy.TurnOffRemote((result) => {
                if (result && result[0]) {
                    this._updateStatus();
                }
            });
        }
    }
    
    _toggleLight(index, state) {
        if (!this._dbusProxy || !this._dbusProxy.TurnOnLightRemote) return;
        
        if (state) {
            this._dbusProxy.TurnOnLightRemote(index, (result) => {
                if (result && result[0]) {
                    this._individualLightSwitches[index].state = true;
                }
            });
        } else {
            this._dbusProxy.TurnOffLightRemote(index, (result) => {
                if (result && result[0]) {
                    this._individualLightSwitches[index].state = false;
                }
            });
        }
    }
    
    _applyScene(scene) {
        if (!this._dbusProxy) return;
        
        this._dbusProxy.ApplySceneRemote(scene.id, (result) => {
            if (result && result[0]) {
                this._currentScene = scene.id;
                this._updateSceneButtons(scene.id);
                
                // Update sliders immediately to match scene
                this._ignoreSliderUpdate = true;
                this._brightnessSlider.value = scene.brightness / 100;
                this._brightnessValue.text = `${scene.brightness}%`;
                this._tempSlider.value = (scene.temperature - 2900) / 4100;
                this._tempValue.text = `${scene.temperature}K`;
                
                let color = this._temperatureToColor(scene.temperature);
                this._tempSlider.style = `color: ${color};`;
                
                this._ignoreSliderUpdate = false;
                
                this._updateStatus();
            }
        });
    }
    
    _updateSceneButtons(activeSceneId) {
        Object.keys(this._sceneButtons).forEach(sceneId => {
            let button = this._sceneButtons[sceneId];
            let scene = SCENES.find(s => s.id === sceneId);
            
            if (sceneId === activeSceneId) {
                button.add_style_class_name('scene-card-active');
                button.style = `
                    background: linear-gradient(135deg, ${scene.color}44, ${scene.color}66);
                    border: 2px solid ${scene.color};
                    padding: 12px;
                    border-radius: 8px;
                    min-width: 140px;
                `;
            } else {
                button.remove_style_class_name('scene-card-active');
                button.style = `
                    background: linear-gradient(135deg, ${scene.color}22, ${scene.color}44);
                    border: 1px solid ${scene.color}66;
                    padding: 12px;
                    border-radius: 8px;
                    min-width: 140px;
                `;
            }
        });
    }
    
    _showDeviceInfo() {
        if (!this._dbusProxy || !this._dbusProxy.GetAccessoryInfoRemote) {
            Main.notify('Ring Light', 'Device info not available (update service)');
            return;
        }
        
        this._dbusProxy.GetAccessoryInfoRemote((result) => {
            if (result) {
                let [productName, firmwareVersion, serialNumber, firmwareBuild, displayName, features] = result;
                
                let message = `Product: ${productName}\n` +
                            `Name: ${displayName}\n` +
                            `Firmware: ${firmwareVersion} (build ${firmwareBuild})\n` +
                            `Serial: ${serialNumber}\n` +
                            `Features: ${features.join(', ')}`;
                
                Main.notify('Ring Light Info', message);
            }
        });
    }
    
    _showPowerOnSettings() {
        if (!this._dbusProxy || !this._dbusProxy.GetSettingsRemote) {
            Main.notify('Ring Light', 'Settings not available (update service)');
            return;
        }
        
        this._dbusProxy.GetSettingsRemote((result) => {
            if (result) {
                let [behavior, brightness, temperature] = result;
                
                let behaviorText = behavior === 1 ? 'Remember Last State' : 'Use Default Settings';
                let message = `Current Power-On Behavior:\n${behaviorText}\n` +
                            `Default Brightness: ${brightness}%\n` +
                            `Default Temperature: ${temperature}K`;
                
                Main.notify('Power-On Settings', message);
            }
        });
    }
    
    _identifyLight() {
        if (!this._dbusProxy) return;
        
        this._dbusProxy.IdentifyRemote((result) => {
            if (result && result[0]) {
                Main.notify('Ring Light', 'Light should be flashing');
            }
        });
    }
    
    _updateStatus() {
        if (!this._dbusProxy) return;
        
        this._dbusProxy.GetStatusRemote((result) => {
            if (result) {
                let [isOn, brightness, temperature] = result;
                this._isOn = isOn;
                this._brightness = brightness;
                this._temperature = temperature;
                
                // Update power switch
                this._powerSwitch.state = isOn;
                
                // Update panel indicator
                this._powerIndicator.style = `
                    width: 6px; 
                    height: 6px; 
                    border-radius: 3px; 
                    background-color: ${isOn ? '#88C0D0' : '#666'}; 
                    margin-left: 2px;
                `;
                
                // Update icon
                this._icon.icon_name = isOn ? 'dialog-information-symbolic' : 'dialog-information-symbolic';
                
                // Update sliders without triggering callbacks
                this._ignoreSliderUpdate = true;
                this._brightnessSlider.value = brightness / 100;
                this._brightnessValue.text = `${brightness}%`;
                this._tempSlider.value = (temperature - 2900) / 4100;
                this._tempValue.text = `${temperature}K`;
                
                let color = this._temperatureToColor(temperature);
                this._tempSlider.style = `color: ${color};`;
                
                this._ignoreSliderUpdate = false;
                
                // Check which scene matches current settings
                let matchingScene = SCENES.find(scene => 
                    Math.abs(scene.brightness - brightness) < 5 &&
                    Math.abs(scene.temperature - temperature) < 200
                );
                this._updateSceneButtons(matchingScene ? matchingScene.id : null);
            }
        });
        
        // Update individual light states if multiple lights
        if (this._numLights > 1 && this._dbusProxy.GetAllLightsStatusRemote) {
            this._dbusProxy.GetAllLightsStatusRemote((result) => {
                if (result) {
                    let [isOnArray] = result;
                    for (let i = 0; i < Math.min(isOnArray.length, this._individualLightSwitches.length); i++) {
                        this._individualLightSwitches[i].state = isOnArray[i];
                    }
                }
            });
        }
    }
    
    destroy() {
        super.destroy();
    }
});

// Extension entry points
let ringLightIndicator;

export default class RingLightExtension {
    constructor(uuid) {
        this._uuid = uuid;
    }
    
    enable() {
        ringLightIndicator = new RingLightIndicator();
        Main.panel.addToStatusArea(this._uuid, ringLightIndicator);
    }
    
    disable() {
        if (ringLightIndicator) {
            ringLightIndicator.destroy();
            ringLightIndicator = null;
        }
    }
}