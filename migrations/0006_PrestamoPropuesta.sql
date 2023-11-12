CREATE TABLE PrestamoPropuesta(
    "fkPrestamo" BIGINT NOT NULL REFERENCES Prestamo(ID),
    "walletId" TEXT NOT NULL,
    "fkUsuario" BIGINT NOT NULL REFERENCES Usuario(ID)
);