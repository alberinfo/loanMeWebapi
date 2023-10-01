CREATE TYPE tiposUsuario AS ENUM('prestatario', 'prestamista', 'administrador');

CREATE TABLE Usuario(
    ID BIGSERIAL PRIMARY KEY NOT NULL,
    email TEXT UNIQUE NOT NULL,
    "nombreCompleto" TEXT NOT NULL,
    "nombreUsuario" TEXT UNIQUE NOT NULL,
    contrasenna TEXT NOT NULL,
    "idWallet" TEXT,
    "tipoUsuario" tiposusuario NOT NULL,
    habilitado BOOL NOT NULL
);