CREATE TABLE PrestamoPropuesta(
    "fkPrestamo" BIGINT NOT NULL REFERENCES Prestamo(ID),
    "fkUsuario" BIGINT NOT NULL REFERENCES Usuario(ID)
);