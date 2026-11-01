"""Génère assets/models/arbre.glb : un arbre « low-poly » au format glTF binaire.

Usage (depuis la racine du projet) :

    python -I tools/generer_arbre.py

Le modèle est volontairement simple, mais il exerce les principaux cas du
chargeur glTF de ZENTRIX (crates/zentrix-assets/src/gltf.rs) :

- une hiérarchie de nœuds : « Arbre » → « Tronc », « Feuillage » (décalé vers le haut) ;
- deux matériaux : une écorce texturée (PNG intégré au fichier) et un
  feuillage d'une seule couleur (baseColorFactor) ;
- le tronc fournit ses normales, ses UV et des indices ;
- le feuillage ne fournit QUE des positions, sans indices : le chargeur doit
  calculer les normales et numéroter les sommets lui-même.

Aucune dépendance : seulement la bibliothèque standard de Python.
Format : https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html
"""

import json
import math
import struct
import zlib
from pathlib import Path

SORTIE = Path(__file__).resolve().parent.parent / "assets" / "models" / "arbre.glb"


# --------------------------------------------------------------------------
# Petits outils vectoriels
# --------------------------------------------------------------------------

def sub(a, b):
    return [a[i] - b[i] for i in range(3)]


def cross(a, b):
    return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]


def dot(a, b):
    return sum(a[i] * b[i] for i in range(3))


def normalize(a):
    n = math.sqrt(dot(a, a))
    return [x / n for x in a]


def oriente(tri, centre):
    """Remet un triangle dans le sens anti-horaire VU DE L'EXTÉRIEUR
    (la face avant pour glTF comme pour ZENTRIX), en comparant sa normale à la
    direction centre → triangle. Valable pour des formes convexes."""
    a, b, c = tri
    n = cross(sub(b, a), sub(c, a))
    milieu = [(a[i] + b[i] + c[i]) / 3 for i in range(3)]
    return tri if dot(n, sub(milieu, centre)) > 0 else [a, c, b]


# --------------------------------------------------------------------------
# Géométrie
# --------------------------------------------------------------------------

def tronc(cotes=6, rayon=0.14, hauteur=1.0):
    """Un prisme hexagonal : normales par face (arêtes vives), UV, indices."""
    positions, normales, uvs, indices = [], [], [], []
    for i in range(cotes):
        a0 = 2 * math.pi * i / cotes
        a1 = 2 * math.pi * (i + 1) / cotes
        p = lambda a, y: [rayon * math.cos(a), y, rayon * math.sin(a)]
        b0, b1, h1, h0 = p(a0, 0.0), p(a1, 0.0), p(a1, hauteur), p(a0, hauteur)
        quad = [b0, b1, h1, h0]
        uv = [[i / cotes, 1.0], [(i + 1) / cotes, 1.0], [(i + 1) / cotes, 0.0], [i / cotes, 0.0]]
        # Sens anti-horaire vu de l'extérieur ? Sinon on inverse le quadrilatère.
        dehors = [math.cos((a0 + a1) / 2), 0.0, math.sin((a0 + a1) / 2)]
        if dot(cross(sub(b1, b0), sub(h1, b0)), dehors) < 0:
            quad.reverse()
            uv.reverse()
        n = dehors
        base = len(positions)
        positions += quad
        normales += [n] * 4
        uvs += uv
        indices += [base, base + 1, base + 2, base, base + 2, base + 3]
    return positions, normales, uvs, indices


def cone(segments, rayon, bas, sommet):
    """Un cône fermé, en triangles NON indexés (3 sommets par triangle)."""
    centre = [0.0, bas + (sommet - bas) / 4, 0.0]  # centre de gravité d'un cône
    pointe = [0.0, sommet, 0.0]
    milieu_base = [0.0, bas, 0.0]
    triangles = []
    for i in range(segments):
        a0 = 2 * math.pi * i / segments
        a1 = 2 * math.pi * (i + 1) / segments
        p0 = [rayon * math.cos(a0), bas, rayon * math.sin(a0)]
        p1 = [rayon * math.cos(a1), bas, rayon * math.sin(a1)]
        triangles.append(oriente([p0, p1, pointe], centre))  # flanc
        triangles.append(oriente([p0, p1, milieu_base], centre))  # fond
    return [p for tri in triangles for p in tri]


def feuillage():
    # Deux cônes empilés, à 8 segments : la silhouette classique d'un sapin.
    return cone(8, 0.75, 0.0, 1.15) + cone(8, 0.52, 0.55, 1.65)


# --------------------------------------------------------------------------
# Texture d'écorce (PNG écrit à la main avec zlib)
# --------------------------------------------------------------------------

def png_ecorce(taille=64):
    lignes = []
    for y in range(taille):
        ligne = bytearray([0])  # filtre PNG « None » en début de ligne
        for x in range(taille):
            # Des stries verticales irrégulières, plus sombres tous les 8 pixels.
            onde = math.sin(x * 0.9 + math.sin(y * 0.35) * 1.5)
            v = 0.55 + 0.25 * onde - (0.25 if x % 8 == 0 else 0.0)
            r, g, b = 0.42 * v, 0.27 * v, 0.15 * v
            ligne += bytes(int(max(0, min(1, c)) * 255) for c in (r, g, b))
        lignes.append(bytes(ligne))

    def bloc(type_, donnees):
        return (struct.pack(">I", len(donnees)) + type_ + donnees
                + struct.pack(">I", zlib.crc32(type_ + donnees) & 0xFFFFFFFF))

    entete = struct.pack(">IIBBBBB", taille, taille, 8, 2, 0, 0, 0)  # RGB 8 bits
    return (b"\x89PNG\r\n\x1a\n" + bloc(b"IHDR", entete)
            + bloc(b"IDAT", zlib.compress(b"".join(lignes), 9)) + bloc(b"IEND", b""))


# --------------------------------------------------------------------------
# Assemblage du glTF binaire (.glb)
# --------------------------------------------------------------------------

class Tampon:
    """Le buffer binaire unique du .glb, et les « bufferViews » / « accessors »
    qui décrivent comment y lire chaque tableau."""

    def __init__(self):
        self.octets = bytearray()
        self.vues = []
        self.accesseurs = []

    def _vue(self, donnees, cible=None):
        while len(self.octets) % 4:  # alignement sur 4 octets exigé par glTF
            self.octets.append(0)
        vue = {"buffer": 0, "byteOffset": len(self.octets), "byteLength": len(donnees)}
        if cible:
            vue["target"] = cible
        self.octets += donnees
        self.vues.append(vue)
        return len(self.vues) - 1

    def flottants(self, vecteurs, type_):
        donnees = b"".join(struct.pack("<%df" % len(v), *v) for v in vecteurs)
        acc = {"bufferView": self._vue(donnees, 34962), "componentType": 5126,
               "count": len(vecteurs), "type": type_}
        if type_ == "VEC3":  # min/max obligatoires pour les positions
            acc["min"] = [min(v[i] for v in vecteurs) for i in range(3)]
            acc["max"] = [max(v[i] for v in vecteurs) for i in range(3)]
        self.accesseurs.append(acc)
        return len(self.accesseurs) - 1

    def indices(self, liste):
        donnees = struct.pack("<%dH" % len(liste), *liste)
        self.accesseurs.append({"bufferView": self._vue(donnees, 34963), "componentType": 5123,
                                "count": len(liste), "type": "SCALAR"})
        return len(self.accesseurs) - 1

    def image(self, png):
        return self._vue(png)


def main():
    t = Tampon()

    pos, nor, uv, idx = tronc()
    tronc_prim = {
        "attributes": {"POSITION": t.flottants(pos, "VEC3"),
                       "NORMAL": t.flottants(nor, "VEC3"),
                       "TEXCOORD_0": t.flottants(uv, "VEC2")},
        "indices": t.indices(idx),
        "material": 0,
    }
    feuillage_prim = {
        "attributes": {"POSITION": t.flottants(feuillage(), "VEC3")},
        "material": 1,
    }
    vue_png = t.image(png_ecorce())

    gltf = {
        "asset": {"version": "2.0", "generator": "ZENTRIX tools/generer_arbre.py"},
        "scene": 0,
        "scenes": [{"name": "Scène", "nodes": [0]}],
        "nodes": [
            {"name": "Arbre", "children": [1, 2]},
            {"name": "Tronc", "mesh": 0},
            {"name": "Feuillage", "mesh": 1, "translation": [0.0, 0.85, 0.0]},
        ],
        "meshes": [
            {"name": "Tronc", "primitives": [tronc_prim]},
            {"name": "Feuillage", "primitives": [feuillage_prim]},
        ],
        "materials": [
            {"name": "Écorce", "pbrMetallicRoughness": {
                "baseColorTexture": {"index": 0}, "metallicFactor": 0.0, "roughnessFactor": 1.0}},
            {"name": "Aiguilles", "pbrMetallicRoughness": {
                "baseColorFactor": [0.16, 0.48, 0.2, 1.0], "metallicFactor": 0.0, "roughnessFactor": 1.0}},
        ],
        "textures": [{"source": 0, "sampler": 0}],
        "samplers": [{"magFilter": 9729, "minFilter": 9987, "wrapS": 10497, "wrapT": 10497}],
        "images": [{"name": "Écorce", "bufferView": vue_png, "mimeType": "image/png"}],
        "buffers": [{"byteLength": len(t.octets)}],
        "bufferViews": t.vues,
        "accessors": t.accesseurs,
    }

    json_octets = json.dumps(gltf, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
    json_octets += b" " * (-len(json_octets) % 4)
    binaire = bytes(t.octets) + b"\0" * (-len(t.octets) % 4)
    longueur = 12 + 8 + len(json_octets) + 8 + len(binaire)
    glb = (struct.pack("<4sII", b"glTF", 2, longueur)
           + struct.pack("<I4s", len(json_octets), b"JSON") + json_octets
           + struct.pack("<I4s", len(binaire), b"BIN\0") + binaire)

    SORTIE.parent.mkdir(parents=True, exist_ok=True)
    SORTIE.write_bytes(glb)
    print(f"{SORTIE} : {len(glb)} octets")


if __name__ == "__main__":
    main()
