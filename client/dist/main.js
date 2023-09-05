const b64ToArrayBuffer = async (data) => {
  return await (
    await fetch(`data:application/octet-binary;base64,${data}`)
  ).arrayBuffer();
};

const arrayBufferToBase64 = (bytes) => {
  return btoa(String.fromCharCode.apply(null, new Uint8Array(bytes)));
};

const parseClientData = (encodedClientData) => {
  return JSON.parse(new TextDecoder().decode(encodedClientData));
};

document
  .getElementById('register')
  .addEventListener('submit', async (event) => {
    event.preventDefault();
    const email = document.getElementById('email').value;
    const response = await fetch(
      `http://localhost:8080/credentials?email=${email}`
    );
    const body = await response.json();
    const challenge = await b64ToArrayBuffer(body.challenge);
    const userId = await b64ToArrayBuffer(body.user.id);
    const createCredentialsOptions = {
      publicKey: {...body, challenge, user: {...body.user, id: userId}},
    };
    console.log(createCredentialsOptions);
    const credentials = await navigator.credentials.create(
      createCredentialsOptions
    );
    const addCredsData = {
      id: credentials.id,
      email,
      response: {
        attestationObject: arrayBufferToBase64(
          credentials.response.attestationObject
        ),
        clientData: parseClientData(credentials.response.clientDataJSON),
      },
    };
    console.log(addCredsData);
    const resp = await fetch('http://localhost:8080/credentials', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(addCredsData),
    });
    const x = await resp.text();
    console.log(x);
  });
