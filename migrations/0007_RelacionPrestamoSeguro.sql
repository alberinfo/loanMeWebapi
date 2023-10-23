CREATE TABLE RelacionPrestamoSeguro(
    "fkPrestamo" BIGINT NOT NULL REFERENCES Prestamo(ID),
    "fkSeguro" BIGINT NOT NULL REFERENCES Seguro(ID)
)