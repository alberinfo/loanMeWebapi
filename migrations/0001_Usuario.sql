CREATE TABLE Usuario(
    ID SERIAL PRIMARY KEY NOT NULL,
    email VARCHAR(50) NOT NULL,
    nombreCompleto VARCHAR(150) NOT NULL,
    nombreUsuario VARCHAR(30) NOT NULL,
    hashContrasenna VARCHAR(86) NOT NULL,
    salt VARCHAR(24) NOT NULL,
    idWallet VARCHAR(200),
    tipoUsuario VARCHAR(30) NOT NULL
);