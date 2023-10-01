CREATE TABLE PerfilCrediticio(
    ID BIGSERIAL PRIMARY KEY NOT NULL,
    "fkUsuario" BIGINT NOT NULL REFERENCES Usuario(ID),
    DNI TEXT UNIQUE NOT NULL,
    "historialCrediticio" TEXT,
    "extractoBancario" TEXT,
    "comprobanteDeIngreso" TEXT,
    "descripcionFinanciera" TEXT
)