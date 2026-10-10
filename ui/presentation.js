(() => {
  const view={compact:false,pinned:true,visible:true,theme:window.clickpulseTheme||'blue'};
  window.presentationView=view;
  const put=(id,value)=>{$(id).textContent=value;};
  const onWindows=window.clickpulsePlatform==='windows';
  if(onWindows||window.clickpulsePlatform==='macos'){
    document.querySelector('header').addEventListener('mousedown',event=>{
      if(event.button!==0||event.target.closest('button,select,input,a,[role="combobox"]'))return;
      event.preventDefault();send('drag');
    });
  }
  if(onWindows){
    const header=document.querySelector('header');
    header.addEventListener('dblclick',event=>{
      if(event.button===0&&!event.target.closest('button,select,input,a,[role="combobox"]'))send('maximize');
    });
    for(const [id,action] of [['window-minimize','minimize'],['window-maximize','maximize'],['window-close','close']])$(id).addEventListener('click',()=>send(action));
    $('close-action-row').hidden=false;
    $('close-action-select').addEventListener('change',event=>send('close_preference',{close_action:event.target.value}));
    const overlay=$('close-overlay'),remember=$('close-remember');
    const surfaces=[document.querySelector('main'),document.querySelector('.sidebar'),$('mini-widget')];
    let previousFocus=null;
    function dismiss(){
      overlay.hidden=true;
      for(const surface of surfaces)surface.inert=false;
      previousFocus?.focus();
    }
    window.requestClose=()=>{
      if(!overlay.hidden)return;
      Dropdowns.closeAll();previousFocus=document.activeElement;
      remember.checked=false;overlay.hidden=false;
      for(const surface of surfaces)surface.inert=true;
      $('close-hide').focus();
    };
    $('close-cancel').addEventListener('click',dismiss);
    for(const [id,action] of [['close-hide','hide'],['close-quit','quit']])$(id).addEventListener('click',()=>{
      const save=remember.checked;dismiss();send('close_decision',{close_action:action,remember:save});
    });
    overlay.addEventListener('keydown',event=>{
      if(event.key==='Escape'){event.preventDefault();dismiss();}
      if(event.key==='Tab'){
        const focusable=[$('close-cancel'),remember,$('close-hide'),$('close-quit')];
        const index=focusable.indexOf(document.activeElement);
        event.preventDefault();focusable[(index+(event.shiftKey?-1:1)+focusable.length)%focusable.length].focus();
      }
    });
  }
  window.receiveView=next=>{
    if(next.locale)window.setInterfaceLanguage(next.locale,next.language_preference);
    if(next.theme)window.setInterfaceTheme(next.theme);
    Object.assign(view,next);
    if(onWindows){
      const key=view.maximized?'window.restore':'window.maximize';
      $('window-maximize').setAttribute('aria-label',t(key));$('window-maximize').title=t(key);
      $('window-maximize-glyph').setAttribute('d',view.maximized?'M5 3h8v8M3 5h8v8H3Z':'M3 3h10v10H3Z');
      if(next.close_action){$('close-action-select').value=next.close_action;Dropdowns.sync('close-action-select');}
    }
    document.body.classList.toggle('compact',view.compact);
    if(view.compact||!view.visible)Dropdowns.closeAll();
    $('mini-widget').hidden=!view.compact;
    $('mini-pin').setAttribute('aria-pressed',String(view.pinned));
    $('mini-pin').setAttribute('aria-label',t(view.pinned?'mini.unpin':'mini.pin'));
    $('mini-pin').title=t(view.pinned?'mini.unpin':'mini.pin');
    if(data)window.receive(data);
  };
  window.presentationStats=snapshot=>{
    const caption=t(snapshot.demo?'common.demo':snapshot.capture.error?'status.recording_error':snapshot.paused?'status.paused':snapshot.source_mode?'status.history':snapshot.capture.listening?'status.recording':'status.permission');
    put('mini-status',caption);
    $('mini-status').classList.toggle('error',!!snapshot.capture.error||snapshot.capture.permission_needed);
    put('mini-today',n(total(snapshot.today)));
    put('mini-keyboard',n(snapshot.today.keyboard));
    put('mini-mouse',n(snapshot.today.mouse));
    $('mini-status').classList.toggle('paused',snapshot.paused);
    put('mini-pause-label',t(snapshot.paused?'mini.resume':'mini.pause'));
    $('mini-pause').setAttribute('aria-label',t(snapshot.source_mode?(snapshot.paused?'action.resume_refresh':'action.pause_refresh'):(snapshot.paused?'action.resume':'action.pause')));
    $('mini-pause').classList.toggle('paused',snapshot.paused);
    $('mini-pause-glyph').setAttribute('d',snapshot.paused?'M5 3l7 5-7 5Z':'M6 4v8m4-8v8');
  };
  $('mini-drag').addEventListener('mousedown',event=>{if(event.button===0){event.preventDefault();send('drag');}});
  for(const [id,action] of [['compact','compact'],['mini-expand','expand'],['mini-hide','hide'],['mini-pin','pin'],['mini-pause','pause']]) {
    $(id).addEventListener('click',()=>send(action));
  }
})();
