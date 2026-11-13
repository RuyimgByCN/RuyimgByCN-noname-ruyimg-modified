({
        thunderIsPhone() {
          //获取浏览器navigator对象的userAgent属性（浏览器用于HTTP请求的用户代理头的值）
          var info = navigator.userAgent;
          //通过正则表达式的test方法判断是否包含“Mobile”字符串
          var isPhone = /mobile|Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|OperaMini/i.test(info);
          //如果包含“Mobile”（是手机设备）则返回true
          return isPhone;
        },
        thunderForbidTouch: function () {
          _status.th_swipe_up = lib.config.swipe_up;
          lib.config.swipe_up = '';
          _status.th_swipe_down = lib.config.swipe_down;
          lib.config.swipe_down = '';
          _status.th_swipe_left = lib.config.swipe_left;
          lib.config.swipe_left = '';
          _status.th_swipe_right = lib.config.swipe_right;
          lib.config.swipe_right = '';
          _status.th_gamePause = ui.click.pause;
          if(ui.roundmenu) ui.roundmenu.hide();
          ui.click.pause = function () { };
          if(_status.auto&&ui.autonode) {
              _status.isHideAutoNode=true;
              ui.autonode.hide();
          }
        },
        thunderAllowTouch: function () {
          if (_status.th_swipe_up) {
            lib.config.swipe_up = _status.th_swipe_up;
            lib.config.swipe_down = _status.th_swipe_down;
            lib.config.swipe_left = _status.th_swipe_left;
            lib.config.swipe_right = _status.th_swipe_right;
            ui.click.pause = _status.th_gamePause;
            //if(ui.roundmenu) ui.roundmenu.show();
          }
          if(ui.roundmenu) ui.roundmenu.show();
          if(_status.isHideAutoNode&&ui.autonode) {
              ui.autonode.show();
          }
          _status.isHideAutoNode=false;
        },
        thunderCreateTimer: function (last, callback, background, width, index, y, theme) {
          var timeTheme;
          if (theme) timeTheme = theme;
          else {
            if (lib.config['extension_雷霆万钧_timeTheme'] == 'follow') {
              if (lib.config['extension_十周年UI_newDecadeStyle'] == 'on') timeTheme = 'decade';
              else timeTheme = 'shousha';
            }
            else timeTheme = lib.config['extension_雷霆万钧_timeTheme'];
          }
          var bg = background || document.body;
          var shoushaJDT = document.getElementById('jindutiao');
          if (shoushaJDT) {
            shoushaJDT.style.cssText += 'transition:none;';
            shoushaJDT.hide();
          }
          if (!last || !callback) return;
          var time, timex;
          if (timeTheme == 'shousha') {
            time = '.th-cxtime', timex = '.th-cxtimecover';
          } else {
            time = '.th-cxtime2', timex = '.th-cxtimecover2';
          }
          let cxTime = ui.create.div(time, bg);            //生成进度条；
          if (width) cxTime.style.setProperty('--w', width + 'px');
          if (index) cxTime.style.zIndex = index;
          if (y) cxTime.style.setProperty('top', 'calc(50% + ' + y + 'px)');
          let cxTimeCover = ui.create.div(timex, cxTime);
          if (width) cxTimeCover.style.setProperty('--w', (width - 4) + 'px');
          else width = 500;
          cxTimeCover.data = last;
          var ct = game.thunderInterval(() => {
            cxTimeCover.data--;
            cxTimeCover.style.width = cxTimeCover.data * Math.round((width - 4) / last * 100) * 0.01 + 'px'
            if (cxTimeCover.data == 0) {
              if (shoushaJDT) shoushaJDT.show();
              callback([cxTime, ct]);
            }
          }, 100); //进度条时间
          return [cxTime, ct];
        },
        thunderInterval(fn, timeout) {
          var timer = {
            thunderFlag: true
          };
          function interval() {
            if (timer.thunderFlag) {
              fn();
              setTimeout(interval, timeout);
            }
          }
          setTimeout(interval, timeout);
          return timer;
        },
        thunderClearInterval(name) {
          if (!name || name == null || typeof name != 'object') return;
          name.thunderFlag = false;
        },
        thunderDialog(player, str, background, height, width, theme, standScale, sswidth) { //触发者  标题  父元素  高度  宽度  指定主题 立绘缩放 手杀主题是否有全屏宽
          var dialogTheme;
          if (theme && lib.config['extension_雷霆万钧_yuqiTheme'] == 'default') dialogTheme = theme;
          else {
            if (lib.config['extension_雷霆万钧_yuqiTheme'] == 'follow') {
              if (lib.config['extension_十周年UI_newDecadeStyle'] == 'on') dialogTheme = 'decade';
              else dialogTheme = 'shousha';
            }
            else dialogTheme = lib.config['extension_雷霆万钧_yuqiTheme'];
          }
          var dialogHeight = height || 400;
          if (dialogTheme == 'decade') dialogHeight = Math.max(290, dialogHeight);
          var canvas = document.createElement('canvas');
          var ctx = canvas.getContext('2d');
          canvas.classList.add('th-thunderDialog');
          canvas.theme = dialogTheme;
          canvas.width = document.body.offsetWidth;
          canvas.height = document.body.offsetHeight;
          canvas.dialogHeight = dialogHeight;
          canvas.onResize = [];
          background.appendChild(canvas);
          var dialogTop = Math.round(canvas.height - dialogHeight) * 0.35;//0.4
          var miniTop = Math.round(canvas.height * 0.6);
          var dialogImg = new Image();
          if (dialogTheme == 'shousha') {
            dialogImg.src = lib.assetURL + 'extension/雷霆万钧/image/effect/ssdialog.png';
            var dialogCanvasWidth = sswidth ? Math.max(width, canvas.width * 0.5) : canvas.width;
            var dialogTap = sswidth ? (canvas.width - dialogCanvasWidth) / 2 : 0;
            var cssWidth = sswidth ? dialogCanvasWidth + 'px' : '100%';
            var cssLeft = sswidth ? 'calc(50% - ' + dialogCanvasWidth / 2 + 'px)' : '0';
            dialogImg.onload = function () {
              ctx.beginPath();
              ctx.drawImage(this, 0, 2, this.width, 50, 0, dialogTop, canvas.width, canvas.width * 0.036);
              ctx.drawImage(this, 0, 54, this.width, 16, dialogTap, dialogTop + canvas.width * 0.036, dialogCanvasWidth, canvas.width * 0.016);
              ctx.drawImage(this, 0, 71, this.width, 20, dialogTap, dialogTop + canvas.width * 0.052, dialogCanvasWidth, dialogHeight);
              ctx.drawImage(this, 0, 92, this.width, 20, dialogTap, dialogTop + canvas.width * 0.052 + dialogHeight, dialogCanvasWidth, canvas.width * 0.011);
              var color = ctx.createLinearGradient(canvas.width * 0.5, dialogTop + 10, canvas.width * 0.5, dialogTop + canvas.width * 0.036 - 20);
              color.addColorStop(0, '#fcfcbf');
              color.addColorStop(1, '#b58b59');
              ctx.font = '40px "th-zhongli"';
              ctx.fillStyle = color;
              // document.fonts.ready.then(function () {
              //ctx.fillText(str, canvas.width * 0.46, dialogTop + 35, 75);
              ctx.fillText(str, canvas.width * 0.46 + 5, dialogTop + 35, 75);
              // })
              /*ctx.moveTo(canvas.width * 0.52 + 5, dialogTop + 17);
              ctx.lineTo(canvas.width * 0.527 + 5, dialogTop + 27);
              ctx.lineTo(canvas.width * 0.534 + 5, dialogTop + 17);*/
              //三角形
              ctx.moveTo(canvas.width * 0.52 + 15, dialogTop + 17 + 3);
              ctx.lineTo(canvas.width * 0.527 + 15, dialogTop + 27 + 3);
              ctx.lineTo(canvas.width * 0.534 + 15, dialogTop + 17 + 3);
              ctx.fill();
              ctx.closePath();
              ctx.beginPath();
              ctx.strokeStyle = 'black';
              // document.fonts.ready.then(function () {
              //ctx.strokeText(str, canvas.width * 0.46, dialogTop + 35, 75);
              ctx.strokeText(str, canvas.width * 0.46 + 5, dialogTop + 35, 75);
              // })
              /*ctx.moveTo(canvas.width * 0.52 + 5, dialogTop + 17);
              ctx.lineTo(canvas.width * 0.527 + 5, dialogTop + 27);
              ctx.lineTo(canvas.width * 0.534 + 5, dialogTop + 17);*/
              //三角形
              ctx.moveTo(canvas.width * 0.52 + 15, dialogTop + 17 + 3);
              ctx.lineTo(canvas.width * 0.527 + 15, dialogTop + 27 + 3);
              ctx.lineTo(canvas.width * 0.534 + 15, dialogTop + 17 + 3);
              ctx.stroke();
              ctx.closePath();
            }
            canvas.minBtn = ui.create.div('.th-thunderDssMin', background);
            canvas.minBtn.style.cssText += 'top:' + (dialogTop + 2) + 'px;';
            canvas.minBtn.addEventListener(game.isMobile(lib.config.touchscreen) ? 'touchstart' : 'mousedown', function (e) {
              e.stopPropagation();
              canvas.resize();
            });
            canvas.container = ui.create.div('.th-thunderDssCon', background);
            canvas.container.style.cssText += 'height:' + dialogHeight + 'px;top:' + (dialogTop + canvas.width * 0.052) + 'px;width:' + cssWidth + ';left:' + cssLeft;
            canvas.resize = function () {
              //暂时关闭先
              return;
              this.height = this.height;
              if (!this.min) { //最小化
                this.min = true;
                this.container.hide();
                this.container.style.pointerEvents = 'none';
                background.style.pointerEvents = 'none';
                this.minBtn.style.top = (miniTop + 2) + 'px';
                ctx.beginPath();
                ctx.drawImage(dialogImg, 0, 2, dialogImg.width, 50, 0, miniTop, this.width, this.width * 0.036);
                var color = ctx.createLinearGradient(this.width * 0.5, miniTop + 10, this.width * 0.5, miniTop + this.width * 0.036 - 20);
                color.addColorStop(0, '#fcfcbf');
                color.addColorStop(1, '#b58b59');
                ctx.font = '40px "th-zhongli"';
                ctx.fillStyle = color;
                ctx.fillText(str, this.width * 0.46 + 5, miniTop + 35);
                ctx.moveTo(this.width * 0.527 + 15, miniTop + 18);
                ctx.lineTo(this.width * 0.52 + 15, miniTop + 28);
                ctx.lineTo(this.width * 0.534 + 15, miniTop + 28);
                ctx.fill();
                ctx.closePath();
                ctx.beginPath();
                ctx.strokeStyle = 'black';
                ctx.strokeText(str, this.width * 0.46 + 5, miniTop + 35);
                ctx.moveTo(this.width * 0.527 + 15, miniTop + 18);
                ctx.lineTo(this.width * 0.52 + 15, miniTop + 28);
                ctx.lineTo(this.width * 0.534 + 15, miniTop + 28);
                ctx.stroke();
                ctx.closePath();
                if(ui.roundmenu) ui.roundmenu.show();
              } else {  //还原
                this.min = false;
                this.container.show();
                this.container.style.pointerEvents = 'auto';
                if (!sswidth) background.style.pointerEvents = 'auto';
                this.minBtn.style.top = (dialogTop + 2) + 'px';
                ctx.beginPath();
                ctx.drawImage(dialogImg, 0, 2, dialogImg.width, 50, 0, dialogTop, this.width, this.width * 0.036);
                ctx.drawImage(dialogImg, 0, 54, dialogImg.width, 16, dialogTap, dialogTop + this.width * 0.036, dialogCanvasWidth, this.width * 0.016);
                ctx.drawImage(dialogImg, 0, 71, dialogImg.width, 20, dialogTap, dialogTop + this.width * 0.052, dialogCanvasWidth, dialogHeight);
                ctx.drawImage(dialogImg, 0, 92, dialogImg.width, 20, dialogTap, dialogTop + this.width * 0.052 + dialogHeight, dialogCanvasWidth, this.width * 0.011);
                var color = ctx.createLinearGradient(this.width * 0.5, dialogTop + 10, this.width * 0.5, dialogTop + this.width * 0.036 - 20);
                color.addColorStop(0, '#fcfcbf');
                color.addColorStop(1, '#b58b59');
                ctx.font = '40px "th-zhongli"';
                ctx.fillStyle = color;
                ctx.fillText(str, this.width * 0.46 + 5, dialogTop + 35, 75);
                ctx.moveTo(this.width * 0.52 + 15, dialogTop + 17 + 3);
                ctx.lineTo(this.width * 0.527 + 15, dialogTop + 27 + 3);
                ctx.lineTo(this.width * 0.534 + 15, dialogTop + 17 + 3);
                ctx.fill();
                ctx.closePath();
                ctx.beginPath();
                ctx.strokeStyle = 'black';
                ctx.strokeText(str, this.width * 0.46 + 5, dialogTop + 35, 75);
                ctx.moveTo(this.width * 0.52 + 15, dialogTop + 17 + 3);
                ctx.lineTo(this.width * 0.527 + 15, dialogTop + 27 + 3);
                ctx.lineTo(this.width * 0.534 + 15, dialogTop + 17 + 3);
                ctx.stroke();
                ctx.closePath();
                if(ui.roundmenu) ui.roundmenu.hide();
              };
              canvas.onResize.forEach(function (fn) {
                fn();
              });
            }
          } else {
            background.style.background = 'rgba(0,0,0,0.6)';
            dialogImg.src = lib.assetURL + 'extension/雷霆万钧/image/effect/buy_dlg.png';
            var dialogWidth;
            if (dialogTheme == 'shousha') dialogWidth = '100%';
            else dialogWidth = width || 800;
            var dialogCanvasWidth = dialogWidth / 0.868;
            dialogImg.onload = function () {
              ctx.beginPath();
              ctx.drawImage(this, 0, 0, this.width, 46, canvas.width * 0.5 - dialogCanvasWidth * 0.5, dialogTop, dialogCanvasWidth, 40);
              ctx.drawImage(this, 0, 47, this.width, 446, canvas.width * 0.5 - dialogCanvasWidth * 0.5, dialogTop + 40, dialogCanvasWidth, dialogHeight);
              ctx.drawImage(this, 0, 493, this.width, 35, canvas.width * 0.5 - dialogCanvasWidth * 0.5, dialogTop + 40 + dialogHeight, dialogCanvasWidth, 20);
              ctx.font = '23px "th-kaishu"';
              ctx.fillStyle = '#281312';
              // document.fonts.ready.then(function () {
              ctx.fillText(str, canvas.width * 0.5 - 23, dialogTop + 30);
              // })
              ctx.closePath();
            }
            canvas.minBtn = ui.create.div('.th-thunderDdcMin', background);
            canvas.minBtn.style.bottom = game.thunderIsPhone() ? 'calc(23% + 33px)' : 'calc(20% + 40px)';
            canvas.minBtn.addEventListener(game.isMobile(lib.config.touchscreen) ? 'touchstart' : 'mousedown', function (e) {
              e.stopPropagation();
              canvas.resize();
            });
            canvas.container = ui.create.div('.th-thunderDdcCon', background);
            canvas.container.style.cssText += 'width:' + dialogWidth + 'px;left:calc(50% - ' + dialogWidth * 0.5 + 'px);height:' + dialogHeight + 'px;top:' + (dialogTop + 40) + 'px;';
            if (player == game.me) game.thunderCreateStand(player, canvas.container, dialogHeight, standScale || 1);
            canvas.resize = function () {
              this.height = this.height;
              if (!this.min) { //最小化
                this.min = true;
                canvas.minBtn.style.backgroundImage = 'url("' + lib.assetURL + 'extension/雷霆万钧/image/effect/showDialog.png")';
                background.style.background = 'rgba(0,0,0,0)';
                this.container.style.pointerEvents = 'none';
                background.style.pointerEvents = 'none';
                this.container.hide();
              } else {  //还原
                this.min = false;
                canvas.minBtn.style.backgroundImage = 'url("' + lib.assetURL + 'extension/雷霆万钧/image/effect/hideDialog.png")';
                background.style.background = 'rgba(0,0,0,0.6)';
                this.container.style.pointerEvents = 'auto';
                background.style.pointerEvents = 'auto';
                this.container.show();
                ctx.beginPath();
                ctx.drawImage(dialogImg, 0, 0, dialogImg.width, 46, canvas.width * 0.5 - dialogCanvasWidth * 0.5, dialogTop, dialogCanvasWidth, 40);
                ctx.drawImage(dialogImg, 0, 47, dialogImg.width, 446, canvas.width * 0.5 - dialogCanvasWidth * 0.5, dialogTop + 40, dialogCanvasWidth, dialogHeight);
                ctx.drawImage(dialogImg, 0, 493, dialogImg.width, 35, canvas.width * 0.5 - dialogCanvasWidth * 0.5, dialogTop + 40 + dialogHeight, dialogCanvasWidth, 20);
                ctx.font = '23px "th-kaishu"';
                ctx.fillStyle = '#281312';
                // document.fonts.ready.then(function () {
                ctx.fillText(str, canvas.width * 0.5 - 23, dialogTop + 30);
                // })
                ctx.closePath();
              };
              canvas.onResize.forEach(function (fn) {
                fn();
              });
            }
          }
          //if(window.shoushaBlanks) window.shoushaBlanks.add(canvas);
          return canvas;
        },
        thunderCreateStand: function (player, dialog, height, y) {
          var jinyu = ui.create.div('.th-jinyu', dialog);
          jinyu.style.setProperty('--h', height + 'px');
          jinyu.style.setProperty('--y', y);

          var playerRealSkin = player.style.backgroundImage;
          if (!playerRealSkin) playerRealSkin = player.childNodes[0].style.backgroundImage;

          playerSkin = playerRealSkin.split('/');
          var skin1 = playerSkin[playerSkin.length - 2], skin2 = playerSkin[playerSkin.length - 1].split('.')[0];
          var reg = new RegExp("[\\u4E00-\\u9FFF]+", "g");
          var skinPath = lib.assetURL + lib.config['extension_雷霆万钧_standPath'];
          if (reg.test(skin2)) {
            skinPath += (skin1 + '-' + skin2 + '.png');
          } else {
            skinPath += (skin2 + '.png')
          }
          if (game.thunderFileExist(skinPath)) jinyu.style.backgroundImage = 'url(' + skinPath + ')';
          else jinyu.style.backgroundImage = playerRealSkin;
        },
        thunderFileExist(url) { //同步
          if (window.XMLHttpRequest) {
            var http = new XMLHttpRequest();
          }
          else {
            var http = new ActiveXObject("Microsoft.XMLHTTP");
          }
          http.open('HEAD', url, false);
          try {
            http.send();
          } catch (err) {
            return false;
          }
          return http.status != 404;
        },

})
