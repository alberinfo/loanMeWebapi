CREATE TABLE Usuario(
    ID SERIAL PRIMARY KEY NOT NULL,
    email TEXT UNIQUE NOT NULL,
    nombreCompleto TEXT NOT NULL,
    nombreUsuario TEXT UNIQUE NOT NULL,
    hashContrasenna TEXT NOT NULL,
    idWallet TEXT,
    tipoUsuario TEXT NOT NULL
);