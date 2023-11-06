CREATE TABLE PrestamoPropuesta(
    "fkPrestamo" BIGINT NOT NULL REFERENCES Prestamo(ID),
    "walletId" TEXT,
    "fkUsuario" BIGINT NOT NULL REFERENCES Usuario(ID)
);