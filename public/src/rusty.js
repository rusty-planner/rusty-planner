"use strict";

(function(w) {
  const loaded = new Promise(function(resolve) {
    w.addEventListener('DOMContentLoaded', resolve, {once:true});
    w.addEventListener('load', resolve, {once:true});
  });
  Object.defineProperty(w, '_load', {
    enumerable: false,
    configurable: false,
    writable: false,
    value: loaded,
  });
})(window);

(async function() {
  console.log('Fetching user...');
  const res = await fetch('/api/user/@me');
  if (res.status === 200) {
    const user = Object.freeze(await res.json());
    window._u = user;
    document.documentElement.setAttribute('data-user', user.id);

    await _load;
    // Setup button
    const profileButton = document.getElementById('user-button');
    if (user.avatar) {
      profileButton.innerHTML = '';
      const avatar = profileButton.appendChild(document.createElement('img'));
      avatar.src = `https://cdn.discordapp.com/avatars/${user.id}/${user.avatar}.webp`;
      avatar.alt = user.name;
    } else {
      profileButton.innerText = user.name;
    }
    profileButton.href = `/user/${user.id}`;


  }
})().then(null, console.error);

(async function(d) {
  await _load;
  console.log('Document ready, registering listeners...');
  document.getElementById('create-group').addEventListener('click', async function() {
    // TODO: Show modal
    const res = await fetch('/api/user/@me/guilds');
    console.log(await res.json());
  });
})(document).then(null, console.error);
