CREATE TABLE RelacionPrestamoSeguro(
    fkPrestamo INT NOT NULL REFERENCES Prestamo(ID),
    fkSeguro INT NOT NULL REFERENCES Seguro(ID)
)