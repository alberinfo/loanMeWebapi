CREATE TABLE Seguro(
	ID BIGSERIAL PRIMARY KEY NOT NULL,
	prima float NOT NULL,
	"plazoPago" DATE NOT NULL,
	"intervaloPago" TEXT NOT NULL,
	"montoACubrir" float NOT NULL
);